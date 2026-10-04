use super::DaemonOverview;

/// Upper bound on the daemon status IPC during a TUI data load. A daemon that
/// accepts the connection but never replies must not freeze startup.
const DAEMON_STATUS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

pub(super) async fn load_daemon_info(context: &crate::app::context::AppContext) -> DaemonOverview {
    let socket = crate::app::daemon::ipc::default_socket_path(&context.runtime_paths.runtime_dir);
    let status = tokio::time::timeout(
        DAEMON_STATUS_TIMEOUT,
        crate::app::daemon::ipc::proxy_status_daemon(&socket),
    )
    .await;
    match status {
        Ok(Ok(response)) => {
            let payload = response.payload;
            DaemonOverview {
                running: payload.as_ref().map(|p| p.daemon_ready).unwrap_or(false),
                rotation_enabled: payload
                    .as_ref()
                    .map(|p| p.rotation_enabled)
                    .unwrap_or(false),
                interval_secs: payload.as_ref().map(|p| p.interval_secs).unwrap_or(0),
            }
        }
        // IPC error or timeout: treat the daemon as unavailable.
        _ => DaemonOverview::default(),
    }
}
