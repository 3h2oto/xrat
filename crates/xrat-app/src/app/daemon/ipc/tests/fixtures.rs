use super::*;

pub(super) fn test_socket_path(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "xrat-daemon-test-{name}-{}-{stamp}.sock",
        std::process::id()
    ))
}

pub(super) async fn wait_until_reachable(socket_path: &Path) {
    for _ in 0..50 {
        if ping_daemon(socket_path).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("daemon socket never became reachable");
}

pub(super) async fn shutdown_test_server(
    socket_path: &Path,
) -> DaemonResponse<DaemonShutdownPayload> {
    daemon_shutdown_daemon(socket_path)
        .await
        .expect("shutdown request should succeed")
}

pub(super) fn spawn_test_supervisor(
    mut rx: mpsc::Receiver<SupervisorEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                SupervisorEvent::DaemonPing { respond_to } => {
                    let _ = respond_to.send(PingPayload {
                        daemon_ready: true,
                        live_tun: true,
                    });
                }
                SupervisorEvent::DaemonShutdown { respond_to } => {
                    let _ = respond_to.send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
                        daemon_ready: false,
                        runtime_disconnected: false,
                    }));
                    break;
                }
                SupervisorEvent::RuntimeReplace {
                    trigger,
                    candidate_id,
                    respond_to,
                } => {
                    let _ = respond_to.send(RuntimeReplaceResult::Ok(RuntimeReplacePayload {
                        trigger,
                        replaced: true,
                        old_session_id: Some(10),
                        new_config_id: candidate_id.unwrap_or(xrat_model::ConfigId(20)),
                        new_session_id: 30,
                        new_pid: 40,
                    }));
                }
                _ => {}
            }
        }
    })
}

pub(super) fn spawn_test_supervisor_replace_error(
    mut rx: mpsc::Receiver<SupervisorEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                SupervisorEvent::DaemonPing { respond_to } => {
                    let _ = respond_to.send(PingPayload {
                        daemon_ready: true,
                        live_tun: true,
                    });
                }
                SupervisorEvent::RuntimeReplace { respond_to, .. } => {
                    let _ = respond_to.send(RuntimeReplaceResult::Err {
                        message: "replace validation failed".to_string(),
                    });
                }
                SupervisorEvent::DaemonShutdown { respond_to } => {
                    let _ = respond_to.send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
                        daemon_ready: false,
                        runtime_disconnected: false,
                    }));
                    break;
                }
                _ => {}
            }
        }
    })
}
