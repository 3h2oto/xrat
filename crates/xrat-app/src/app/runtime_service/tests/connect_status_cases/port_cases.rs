use super::support::{import_hy2_config, import_single_config};
use super::*;
use async_trait::async_trait;
use std::ffi::OsString;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Output};
use std::sync::{Arc, Mutex};
use xrat_support::process::{Child, ChildHandle, CommandSpec, ProcessSpawner, Stdio};
use xrat_support::readiness::{
    ChildPollErrorPolicy, NetworkEndpoint, PortWaiter, ReadinessError, ReadinessRequest,
    RuntimeProcessPorts, TcpConnector,
};
use xrat_support::signals::{ProcessSignal, ProcessSignals};

#[derive(Default)]
struct State {
    commands: Vec<Vec<OsString>>,
    spawned: usize,
    killed: usize,
    reaped: usize,
    running: bool,
    signals: Vec<ProcessSignal>,
    connected: Vec<NetworkEndpoint>,
    readiness: usize,
}
struct FakePorts {
    state: Arc<Mutex<State>>,
    fail_readiness: bool,
    fail_validation: bool,
}
struct FakeChild(Arc<Mutex<State>>);
impl ChildHandle for FakeChild {
    fn id(&self) -> u32 {
        42
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Ok(None)
    }
    fn kill(&mut self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.killed += 1;
        state.running = false;
        Ok(())
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.0.lock().unwrap().reaped += 1;
        Ok(ExitStatus::from_raw(0))
    }
}
#[async_trait]
impl ProcessSpawner for FakePorts {
    fn spawn(&self, spec: &CommandSpec) -> io::Result<Child> {
        assert!(matches!(spec.stdin, Some(Stdio::Null)));
        assert!(matches!(spec.stdout, Some(Stdio::File(_))));
        assert!(matches!(spec.stderr, Some(Stdio::File(_))));
        assert!(!spec.kill_on_drop);
        assert_eq!(spec.args[0], "run");
        assert_eq!(spec.args[1], "-c");
        assert!(std::path::Path::new(&spec.args[2]).exists());
        let mut state = self.state.lock().unwrap();
        state.spawned += 1;
        state.running = true;
        state.commands.push(spec.args.clone());
        Ok(Child::from_handle(
            Box::new(FakeChild(self.state.clone())),
            None,
            None,
        ))
    }
    fn run(&self, spec: &CommandSpec, capture: bool) -> io::Result<Output> {
        assert!(capture);
        self.state.lock().unwrap().commands.push(spec.args.clone());
        let version = spec.args.first().is_some_and(|arg| arg == "version");
        if !version {
            assert!(matches!(spec.stdin, Some(Stdio::Null)));
            assert!(spec.args.iter().any(|arg| arg == "-test" || arg == "check"));
        }
        Ok(Output {
            status: ExitStatus::from_raw(if !version && self.fail_validation {
                7 << 8
            } else {
                0
            }),
            stdout: if spec.args.iter().any(|arg| arg == "--name") {
                b"1.13.21".to_vec()
            } else {
                b"Xray 26.7.28".to_vec()
            },
            stderr: if self.fail_validation {
                b"invalid fixture".to_vec()
            } else {
                Vec::new()
            },
        })
    }
    async fn output_async(&self, spec: &CommandSpec) -> io::Result<Output> {
        self.run(spec, true)
    }
}
#[async_trait]
impl PortWaiter for FakePorts {
    async fn wait(
        &self,
        child: &mut dyn ChildHandle,
        request: ReadinessRequest,
    ) -> Result<(), ReadinessError> {
        assert_eq!(child.id(), 42);
        assert!(matches!(request.child_errors, ChildPollErrorPolicy::Fail));
        assert_eq!(request.endpoints.len(), 1);
        self.state.lock().unwrap().readiness += 1;
        if self.fail_readiness {
            Err(ReadinessError::Timeout {
                port: request.endpoints[0].port,
            })
        } else {
            Ok(())
        }
    }
}
impl ProcessSignals for FakePorts {
    fn send(&self, pid: i64, signal: ProcessSignal) -> io::Result<bool> {
        assert_eq!(pid, 42);
        let mut state = self.state.lock().unwrap();
        state.signals.push(signal);
        let running = state.running;
        if signal == ProcessSignal::Term || signal == ProcessSignal::Kill {
            state.running = false;
        }
        Ok(running)
    }
}
#[async_trait]
impl TcpConnector for FakePorts {
    async fn connect(&self, endpoint: &NetworkEndpoint) -> io::Result<()> {
        self.state.lock().unwrap().connected.push(endpoint.clone());
        Ok(())
    }
}
fn ports(fail_readiness: bool, fail_validation: bool) -> (RuntimeProcessPorts, Arc<Mutex<State>>) {
    let state = Arc::new(Mutex::new(State::default()));
    let fake = Arc::new(FakePorts {
        state: state.clone(),
        fail_readiness,
        fail_validation,
    });
    (
        RuntimeProcessPorts {
            spawner: fake.clone(),
            waiter: fake.clone(),
            signals: fake.clone(),
            connector: fake,
            tun: Arc::new(xrat_support::net::SystemTunInterfaceOps),
        },
        state,
    )
}

#[tokio::test]
async fn injected_runtime_ports_cover_preflight_start_status_and_stop_for_both_engines() {
    for singbox in [false, true] {
        let mut context = test_context().await;
        let config = if singbox {
            context.app_config.runtime.engine = "sing-box".into();
            import_hy2_config(&context).await
        } else {
            import_single_config(&context).await
        };
        let (ports, state) = ports(false, false);
        let service = RuntimeService::with_process_ports(&context, ports);
        let connected = service
            .connect(ConnectRequest {
                config_id: config.id,
            })
            .await
            .unwrap();
        assert_eq!(connected.pid, 42);
        let snapshot = service.status().await.unwrap();
        assert!(snapshot.pid_running);
        assert!(!snapshot.inbound_health.has_unreachable_endpoint());
        assert!(service.disconnect().await.unwrap().stopped_session);
        let latest = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
        assert!(context.db.get_active_config().await.unwrap().is_none());
        let state = state.lock().unwrap();
        assert_eq!(state.spawned, 1);
        assert_eq!(state.readiness, 1);
        assert!(!state.connected.is_empty());
        assert!(state.signals.contains(&ProcessSignal::Term));
        assert_eq!(
            state.killed, 0,
            "managed success must detach child ownership"
        );
        assert_eq!(state.reaped, 0);
    }
}

#[tokio::test]
async fn failed_injected_startup_reaps_child_and_persists_failed_session() {
    for singbox in [false, true] {
        let mut context = test_context().await;
        let config = if singbox {
            context.app_config.runtime.engine = "sing-box".into();
            import_hy2_config(&context).await
        } else {
            import_single_config(&context).await
        };
        let (ports, state) = ports(true, false);
        assert!(
            RuntimeService::with_process_ports(&context, ports)
                .connect(ConnectRequest {
                    config_id: config.id
                })
                .await
                .is_err()
        );
        let latest = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.status, RuntimeSessionStatus::Failed);
        assert!(context.db.get_active_config().await.unwrap().is_none());
        let state = state.lock().unwrap();
        assert_eq!((state.spawned, state.killed, state.reaped), (1, 1, 1));
    }
}

#[tokio::test]
async fn injected_preflight_failure_does_not_spawn_or_create_session() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    let (ports, state) = ports(false, true);
    let error = RuntimeService::with_process_ports(&context, ports)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("invalid fixture"));
    assert_eq!(state.lock().unwrap().spawned, 0);
    assert!(
        context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .is_none()
    );
}
