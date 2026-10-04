#![cfg(unix)]
use async_trait::async_trait;
use std::io::{self, Cursor};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{ExitStatus, Output};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use xrat_engines::{singbox, xray};
use xrat_support::process::{Child, ChildHandle, CommandSpec, ProcessSpawner, Stdio};
use xrat_support::readiness::{
    ChildPollErrorPolicy, PortWaiter, ReadinessError, ReadinessRequest, RuntimeProcessPorts,
};

#[derive(Default)]
struct State {
    killed: usize,
    reaped: usize,
    ready: usize,
}
#[derive(Clone, Copy)]
enum Startup {
    Ready,
    Exited,
    Timeout,
    InspectFailed,
    Missing,
    Stalled,
}
struct Fixture {
    state: Arc<Mutex<State>>,
    startup: Startup,
    managed: bool,
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
        self.0.lock().unwrap().killed += 1;
        Ok(())
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.0.lock().unwrap().reaped += 1;
        Ok(ExitStatus::from_raw(0))
    }
}
#[async_trait]
impl ProcessSpawner for Fixture {
    fn spawn(&self, spec: &CommandSpec) -> io::Result<Child> {
        assert_eq!(spec.program, "missing-fixture-engine");
        assert_eq!(spec.args[0], "run");
        assert_eq!(spec.args[1], "-c");
        assert!(Path::new(&spec.args[2]).exists());
        assert!(matches!(spec.stdin, Some(Stdio::Null)));
        if self.managed {
            assert!(matches!(spec.stdout, Some(Stdio::File(_))));
            assert!(matches!(spec.stderr, Some(Stdio::File(_))));
        } else {
            assert!(matches!(spec.stdout, Some(Stdio::Piped)));
            assert!(matches!(spec.stderr, Some(Stdio::Piped)));
        }
        if matches!(self.startup, Startup::Missing) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "fixture missing"));
        }
        Ok(Child::from_handle(
            Box::new(FakeChild(self.state.clone())),
            None,
            Some(Box::new(Cursor::new(b"fixture early exit".to_vec()))),
        ))
    }
    fn run(&self, _: &CommandSpec, _: bool) -> io::Result<Output> {
        panic!("startup must spawn")
    }
    async fn output_async(&self, _: &CommandSpec) -> io::Result<Output> {
        panic!("startup must spawn")
    }
}
#[async_trait]
impl PortWaiter for Fixture {
    async fn wait(
        &self,
        child: &mut dyn ChildHandle,
        request: ReadinessRequest,
    ) -> Result<(), ReadinessError> {
        assert_eq!(child.id(), 42);
        assert_eq!(request.endpoints[0].port, 12345);
        assert_eq!(request.poll_interval, Duration::from_millis(100));
        assert_eq!(request.timeout, Duration::from_secs(1));
        assert_eq!(
            matches!(request.child_errors, ChildPollErrorPolicy::Fail),
            self.managed
        );
        self.state.lock().unwrap().ready += 1;
        match self.startup {
            Startup::Ready => Ok(()),
            Startup::Exited => Err(ReadinessError::ProcessExited(ExitStatus::from_raw(7 << 8))),
            Startup::Timeout => Err(ReadinessError::Timeout { port: 12345 }),
            Startup::InspectFailed => {
                Err(ReadinessError::Io(io::Error::other("fixture inspection")))
            }
            Startup::Missing => panic!("missing process cannot wait"),
            Startup::Stalled => std::future::pending().await,
        }
    }
}
fn ports(startup: Startup, managed: bool) -> (RuntimeProcessPorts, Arc<Mutex<State>>) {
    let state = Arc::new(Mutex::new(State::default()));
    let fixture = Arc::new(Fixture {
        state: state.clone(),
        startup,
        managed,
    });
    (
        RuntimeProcessPorts {
            spawner: fixture.clone(),
            waiter: fixture,
            ..Default::default()
        },
        state,
    )
}
fn xray_config() -> xray::XrayConfig {
    serde_json::from_value(serde_json::json!({"log":{"loglevel":"warning"},"inbounds":[{"tag":"in","port":12345,"listen":"127.0.0.1","protocol":"socks"}],"outbounds":[]})).unwrap()
}
fn singbox_config() -> singbox::SingboxConfig {
    serde_json::from_value(serde_json::json!({"log":{"level":"warn","timestamp":true},"inbounds":[{"type":"socks","tag":"in","listen":"127.0.0.1","listen_port":12345}],"outbounds":[]})).unwrap()
}

#[tokio::test]
async fn both_managed_engines_preserve_startup_errors_and_failed_child_cleanup() {
    for startup in [
        Startup::Ready,
        Startup::Exited,
        Startup::Timeout,
        Startup::InspectFailed,
        Startup::Missing,
    ] {
        let root = tempfile::tempdir().unwrap();
        for singbox in [false, true] {
            let (ports, state) = ports(startup, true);
            let result = if singbox {
                singbox::process_mgmt::spawn_detached_with_ports(
                    Path::new("missing-fixture-engine"),
                    root.path(),
                    1,
                    &singbox_config(),
                    "127.0.0.1",
                    12345,
                    Duration::from_secs(1),
                    ports,
                )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
            } else {
                xray::runtime_process::spawn_detached_with_ports(
                    Path::new("missing-fixture-engine"),
                    root.path(),
                    1,
                    &xray_config(),
                    "127.0.0.1",
                    12345,
                    Duration::from_secs(1),
                    ports,
                )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
            };
            let state = state.lock().unwrap();
            match startup {
                Startup::Ready => {
                    result.unwrap();
                    assert_eq!((state.killed, state.reaped), (0, 0));
                }
                Startup::Missing => {
                    assert!(result.unwrap_err().contains("fixture missing"));
                    assert_eq!((state.ready, state.killed, state.reaped), (0, 0, 0));
                }
                _ => {
                    let error = result.unwrap_err();
                    let expected = match startup {
                        Startup::Exited => "exited during startup",
                        Startup::Timeout => "startup timeout",
                        Startup::InspectFailed => "fixture inspection",
                        _ => unreachable!(),
                    };
                    assert!(error.contains(expected), "{error}");
                    assert_eq!((state.killed, state.reaped), (1, 1));
                }
            }
        }
    }
}

#[tokio::test]
async fn both_temporary_engines_preserve_errors_and_drop_reaps_owned_children() {
    for startup in [
        Startup::Ready,
        Startup::Exited,
        Startup::Timeout,
        Startup::InspectFailed,
        Startup::Missing,
    ] {
        for singbox in [false, true] {
            let (ports, state) = ports(startup, false);
            let result = if singbox {
                singbox::SingboxProbeProcess::spawn_with_binary_with_ports(
                    Path::new("missing-fixture-engine"),
                    &singbox_config(),
                    12345,
                    Duration::from_secs(1),
                    ports,
                )
                .await
                .map(|process| {
                    let path = process.config_path();
                    assert!(path.exists());
                    drop(process);
                    assert!(!path.exists());
                })
                .map_err(|e| e.to_string())
            } else {
                xray::probe_process::XrayProcess::spawn_with_binary_with_ports(
                    Path::new("missing-fixture-engine"),
                    &xray_config(),
                    Duration::from_secs(1),
                    ports,
                )
                .await
                .map(|process| {
                    let path = process.config_path();
                    assert!(path.exists());
                    drop(process);
                    assert!(!path.exists());
                })
                .map_err(|e| e.to_string())
            };
            let state = state.lock().unwrap();
            match startup {
                Startup::Ready => {
                    result.unwrap();
                    assert_eq!((state.killed, state.reaped), (1, 1));
                }
                Startup::Missing => {
                    assert!(result.unwrap_err().contains("fixture missing"));
                    assert_eq!((state.ready, state.killed, state.reaped), (0, 0, 0));
                }
                _ => {
                    let error = result.unwrap_err();
                    if matches!(startup, Startup::Exited) {
                        assert!(error.contains("fixture early exit"), "{error}");
                    } else {
                        assert!(error.contains("not ready"), "{error}");
                    }
                    assert!(state.killed >= 1);
                    assert_eq!(state.reaped, 1);
                }
            }
        }
    }
}

#[tokio::test]
async fn cancelling_startup_reaps_managed_and_temporary_children_for_both_engines() {
    for managed in [false, true] {
        for singbox in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let (ports, state) = ports(Startup::Stalled, managed);
            let mut startup = Box::pin(async {
                match (managed, singbox) {
                    (true, true) => singbox::process_mgmt::spawn_detached_with_ports(
                        Path::new("missing-fixture-engine"),
                        root.path(),
                        1,
                        &singbox_config(),
                        "127.0.0.1",
                        12345,
                        Duration::from_secs(1),
                        ports,
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                    (true, false) => xray::runtime_process::spawn_detached_with_ports(
                        Path::new("missing-fixture-engine"),
                        root.path(),
                        1,
                        &xray_config(),
                        "127.0.0.1",
                        12345,
                        Duration::from_secs(1),
                        ports,
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                    (false, true) => singbox::SingboxProbeProcess::spawn_with_binary_with_ports(
                        Path::new("missing-fixture-engine"),
                        &singbox_config(),
                        12345,
                        Duration::from_secs(1),
                        ports,
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                    (false, false) => xray::XrayProcess::spawn_with_binary_with_ports(
                        Path::new("missing-fixture-engine"),
                        &xray_config(),
                        Duration::from_secs(1),
                        ports,
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                }
            });
            tokio::select! {
                result = &mut startup => panic!("stalled startup unexpectedly returned: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(1)) => {},
            }
            assert_eq!(state.lock().unwrap().ready, 1);
            drop(startup);
            let state = state.lock().unwrap();
            assert_eq!((state.killed, state.reaped), (1, 1));
        }
    }
}
