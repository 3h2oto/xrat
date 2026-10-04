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
