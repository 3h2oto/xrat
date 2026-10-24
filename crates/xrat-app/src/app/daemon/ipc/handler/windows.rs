use tokio::io::AsyncReadExt;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio::sync::mpsc;

use crate::app::daemon::ipc::{
    self, DaemonRequest, DaemonResponse, DaemonResponseCode, PROTOCOL_VERSION,
};
use crate::app::daemon::supervisor::SupervisorEvent;

pub async fn serve_ping(
    endpoint: &std::path::Path,
    supervisor: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<()> {
    let mut listener = ipc::windows::listener(endpoint, true)?;
    let (shutdown, mut receiver) = mpsc::channel(1);
    let signal = xrat_support::signals::wait_for_shutdown();
    tokio::pin!(signal);
    let mut connections = tokio::task::JoinSet::new();
    let capacity = std::sync::Arc::new(tokio::sync::Semaphore::new(64));
    loop {
        tokio::select! {
            biased;
            _ = receiver.recv() => break,
            result = &mut signal => {
                result?;
                ipc::transport::daemon_shutdown_response_via_supervisor(supervisor.clone()).await?;
                break;
            }
            result = listener.connect() => {
                result?;
                let next = ipc::windows::listener(endpoint, false)?;
                let stream = std::mem::replace(&mut listener, next);
                let Ok(permit) = capacity.clone().try_acquire_owned() else { continue; };
                let supervisor = supervisor.clone();
                let shutdown = shutdown.clone();
                connections.spawn(async move {
                    let _permit = permit;
                    match tokio::time::timeout(std::time::Duration::from_secs(30), handle(stream, supervisor, shutdown)).await {
                        Ok(Ok(())) => {},
                        Ok(Err(error)) => tracing::debug!(%error, "daemon pipe request failed"),
                        Err(_) => tracing::debug!("daemon pipe request timed out"),
                    }
                });
            }
            Some(result) = connections.join_next(), if !connections.is_empty() => {
                if let Err(error) = result { tracing::debug!(%error, "daemon pipe task failed"); }
            }
        }
    }
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    Ok(())
}

async fn handle(
    mut stream: NamedPipeServer,
    supervisor: mpsc::Sender<SupervisorEvent>,
    shutdown: mpsc::Sender<()>,
) -> crate::app::Result<()> {
    ipc::windows::verify_client(&stream)?;
    let bytes = ipc::frame::read(&mut stream).await?;
    let request: DaemonRequest = serde_json::from_slice(&bytes)?;
    if request.protocol_version != PROTOCOL_VERSION {
        let response = DaemonResponse::<()> {
            protocol_version: PROTOCOL_VERSION,
            ok: false,
            code: DaemonResponseCode::InvalidState,
            message: format!(
                "unsupported protocol version {} (expected {})",
                request.protocol_version, PROTOCOL_VERSION
            ),
            payload: None,
        };
        ipc::frame::write(&mut stream, &serde_json::to_vec(&response)?).await?;
        let _ = stream.read_u8().await?;
        return Ok(());
    }
    let (response, should_shutdown) =
        super::dispatch::dispatch_request(request.request, supervisor).await?;
    ipc::frame::write(&mut stream, &response).await?;
    let _ = stream.read_u8().await?;
    if should_shutdown {
        let _ = shutdown.send(()).await;
    }
    Ok(())
}
