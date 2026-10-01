use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::StatusArgs;

mod display;

pub async fn run(context: &AppContext, args: &StatusArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);
    match ipc::runtime_status_daemon(&socket_path).await {
        Ok(response) => display::print_daemon_status(context, response, args.json).await,
        Err(err) if ipc::daemon_unreachable(&err) => {
            Err(crate::app::AppError::InvalidArgument(format!(
                "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                socket_path.display()
            )))
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppError;

    #[tokio::test]
    async fn status_returns_daemon_unreachable_hint() {
        let context = test_context("status-daemon-hint").await;
        let err = run(&context, &StatusArgs { json: false })
            .await
            .expect_err("status should require daemon reachability");
        match err {
            AppError::InvalidArgument(message) => {
                assert!(message.contains("xrat daemon start"));
                assert!(message.contains("daemon is not running"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    async fn test_context(prefix: &str) -> AppContext {
        crate::app::tests::TestAppBuilder::new(prefix).build().await
    }
}
