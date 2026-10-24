use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::{TuiImport, run_import};
use crate::app::context::AppContext;

async fn test_context() -> (TempDir, AppContext) {
    let (context, root) = crate::app::tests::TestAppBuilder::new("tui-import")
        .build_with_root()
        .await;
    (root, context)
}

async fn subscription_url(request_count: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    tokio::spawn(async move {
        let body = "vless://uuid-123@example.com:443#One";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        for _ in 0..request_count {
            let (mut socket, _) = listener.accept().await.expect("request should connect");
            // The request has to be drained before the socket is closed.
            // Windows answers an unread request with a reset, which would
            // abort the response the client is still reading.
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = socket
                    .read(&mut chunk)
                    .await
                    .expect("request should be readable");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
            }
            socket
                .write_all(response.as_bytes())
                .await
                .expect("response should be written");
        }
    });
    format!("http://{address}/subscription")
}

#[tokio::test]
async fn config_import_persists_one_config() {
    let (_root, context) = test_context().await;
    let (source, node) =
        crate::app::import::load_single_node("vless://uuid-123@example.com:443#One")
            .expect("config should parse");

    let message = run_import(
        &context,
        TuiImport::Config {
            source,
            node: Box::new(node),
        },
    )
    .await
    .expect("config import should succeed");

    assert_eq!(message, "added 1 config");
    assert_eq!(context.db.get_config_count().await.expect("count"), 1);
}

#[tokio::test]
async fn subscription_import_persists_name_and_reuses_url() {
    let (_root, context) = test_context().await;
    let url = subscription_url(2).await;

    for name in ["First", "Second"] {
        run_import(
            &context,
            TuiImport::Subscription {
                url: url.clone(),
                name: name.to_string(),
            },
        )
        .await
        .expect("subscription import should succeed");
    }

    let subscriptions = context
        .db
        .list_subscriptions()
        .await
        .expect("subscriptions should load");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].name.as_deref(), Some("Second"));
    assert_eq!(context.db.get_config_count().await.expect("count"), 1);
}
