use std::sync::Arc;

use crate::app::context::AppContext;
use crate::app::daemon::ipc::{self};

use super::control::RuntimeControl;
use super::daemon::DaemonRuntimeControl;
use super::local::LocalRuntimeControl;

/// CLI operations require the daemon and never fall back to a second owner.
pub fn daemon_control(context: &AppContext) -> Arc<dyn RuntimeControl> {
    Arc::new(DaemonRuntimeControl::new(ipc::default_socket_path(
        &context.runtime_paths.runtime_dir,
    )))
}

/// Build an in-process control implementation.
pub fn local_control(context: &AppContext) -> Arc<dyn RuntimeControl> {
    Arc::new(LocalRuntimeControl::new(context.clone()))
}

/// TUI operations use the daemon when reachable, or run standalone when absent.
/// Protocol failures and timeouts must not create a competing local owner.
pub async fn tui_control(context: &AppContext) -> crate::app::Result<Arc<dyn RuntimeControl>> {
    #[cfg(unix)]
    if tui_uses_daemon(&ipc::default_socket_path(
        &context.runtime_paths.runtime_dir,
    ))
    .await?
    {
        return Ok(daemon_control(context));
    }
    Ok(local_control(context))
}

#[cfg(unix)]
pub(crate) async fn tui_uses_daemon(socket: &std::path::Path) -> crate::app::Result<bool> {
    let response =
        tokio::time::timeout(std::time::Duration::from_secs(1), ipc::ping_daemon(socket))
            .await
            .map_err(|_| crate::app::AppError::InvalidArgument("daemon ping timed out".into()))?;
    match response {
        Ok(response) if response.ok && response.protocol_version == ipc::PROTOCOL_VERSION => {
            Ok(true)
        }
        Ok(response) => Err(crate::app::AppError::InvalidArgument(format!(
            "daemon unavailable: {}",
            response.message
        ))),
        Err(error) if ipc::daemon_unreachable(&error) => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn absent_daemon_allows_standalone_tui() {
        let root = tempfile::tempdir().unwrap();
        assert!(
            !tui_uses_daemon(&root.path().join("absent.sock"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn reachable_daemon_owns_tui_operations() {
        let root = tempfile::tempdir().unwrap();
        let socket = root.path().join("daemon.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            stream.read_to_end(&mut request).await.unwrap();
            let request: ipc::DaemonRequest = serde_json::from_slice(&request).unwrap();
            assert!(matches!(
                request.request,
                ipc::DaemonRequestKind::DaemonPing
            ));
            stream
                .write_all(&serde_json::to_vec(&ipc::ping_response()).unwrap())
                .await
                .unwrap();
        });
        assert!(tui_uses_daemon(&socket).await.unwrap());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn invalid_daemon_response_does_not_fall_back_to_local() {
        let root = tempfile::tempdir().unwrap();
        let socket = root.path().join("daemon.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            stream.read_to_end(&mut request).await.unwrap();
            stream.write_all(b"invalid response").await.unwrap();
        });
        assert!(tui_uses_daemon(&socket).await.is_err());
        server.await.unwrap();
    }
}
