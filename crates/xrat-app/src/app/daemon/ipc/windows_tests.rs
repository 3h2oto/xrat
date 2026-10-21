use super::*;
use crate::app::daemon::supervisor::{DaemonShutdownResult, SupervisorEvent};
use tokio::io::AsyncWriteExt;

async fn connect(endpoint: &std::path::Path) -> tokio::net::windows::named_pipe::NamedPipeClient {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            match windows::connect(endpoint) {
                Ok(pipe) => return pipe,
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(231) =>
                {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                Err(error) => panic!("pipe connection failed: {error}"),
            }
        }
    })
    .await
    .expect("pipe connection timed out")
}

fn endpoint() -> PathBuf {
    default_socket_path(&std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()))
}

#[test]
fn pipe_names_are_stable_and_profile_isolated() {
    let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    assert_eq!(default_socket_path(&root), default_socket_path(&root));
    assert_ne!(
        default_socket_path(&root),
        default_socket_path(&root.join("other"))
    );
    assert!(
        default_socket_path(&root)
            .to_string_lossy()
            .starts_with(r"\\.\pipe\xrat-")
    );
}

#[tokio::test]
async fn refuses_duplicate_pipe_owners() {
    let endpoint = endpoint();
    let _owner = windows::listener(&endpoint, true).unwrap();
    assert!(windows::listener(&endpoint, true).is_err());
}

#[tokio::test]
async fn native_ping_and_shutdown_use_the_shared_supervisor() {
    let endpoint = endpoint();
    let (supervisor, mut receiver) = tokio::sync::mpsc::channel(8);
    let worker = tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            match event {
                SupervisorEvent::DaemonPing { respond_to } => {
                    respond_to
                        .send(PingPayload {
                            daemon_ready: true,
                            live_tun: true,
                        })
                        .unwrap();
                }
                SupervisorEvent::DaemonShutdown { respond_to } => {
                    respond_to
                        .send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
                            daemon_ready: false,
                            runtime_disconnected: true,
                        }))
                        .unwrap();
                    break;
                }
                _ => panic!("unexpected supervisor request"),
            }
        }
    });
    let server_endpoint = endpoint.clone();
    let server = tokio::spawn(async move { serve_ping(&server_endpoint, supervisor).await });
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Ok(response) = ping_daemon(&endpoint).await {
            assert!(response.ok);
            assert!(response.payload.unwrap().live_tun);
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "pipe did not become ready"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(
        daemon_shutdown_daemon(&endpoint)
            .await
            .unwrap()
            .payload
            .unwrap()
            .runtime_disconnected
    );
    tokio::time::timeout(std::time::Duration::from_secs(5), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn mismatched_protocol_and_malformed_frames_do_not_poison_the_listener() {
    let endpoint = endpoint();
    let (supervisor, _receiver) = tokio::sync::mpsc::channel(8);
    let server_endpoint = endpoint.clone();
    let server = tokio::spawn(async move { serve_ping(&server_endpoint, supervisor).await });
    let mut pipe = connect(&endpoint).await;
    let request = DaemonRequest {
        protocol_version: PROTOCOL_VERSION + 1,
        request: DaemonRequestKind::DaemonPing,
    };
    frame::write(&mut pipe, &serde_json::to_vec(&request).unwrap())
        .await
        .unwrap();
    let response: DaemonResponse<()> =
        serde_json::from_slice(&frame::read(&mut pipe).await.unwrap()).unwrap();
    pipe.write_all(&[0]).await.unwrap();
    assert!(!response.ok);
    assert!(response.message.contains("unsupported protocol"));
    drop(pipe);
    let mut invalid = connect(&endpoint).await;
    invalid.write_all(&u32::MAX.to_le_bytes()).await.unwrap();
    drop(invalid);
    let mut recovered = connect(&endpoint).await;
    frame::write(&mut recovered, &serde_json::to_vec(&request).unwrap())
        .await
        .unwrap();
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        frame::read(&mut recovered),
    )
    .await
    .unwrap()
    .unwrap();
    let response: DaemonResponse<()> = serde_json::from_slice(&response).unwrap();
    assert!(!response.ok);
    recovered.write_all(&[0]).await.unwrap();
    server.abort();
    let _ = server.await;
}
