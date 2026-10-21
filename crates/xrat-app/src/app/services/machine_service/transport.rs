use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio::sync::{Semaphore, mpsc};

use super::{
    ENDPOINT, GATEWAY_VERSION, GatewayPayload, GatewayRequest, MachineGateway, MachineProfile,
};
use crate::app::daemon::{ipc, supervisor::SupervisorEvent};
use crate::app::{AppError, Result};

pub async fn serve(
    profile: MachineProfile,
    supervisor: mpsc::Sender<SupervisorEvent>,
    shutdown: xrat_support::cancel::CancellationReceiver,
) -> Result<()> {
    serve_with_ready(profile, supervisor, shutdown, None).await
}

pub(super) async fn serve_with_ready(
    profile: MachineProfile,
    supervisor: mpsc::Sender<SupervisorEvent>,
    shutdown: xrat_support::cancel::CancellationReceiver,
    ready: Option<tokio::sync::oneshot::Sender<()>>,
) -> Result<()> {
    let gateway = Arc::new(MachineGateway::new(&profile, supervisor)?);
    let endpoint = std::path::Path::new(ENDPOINT);
    let mut listener = ipc::windows::listener_for_owner(endpoint, true, profile.operator_sid())?;
    if let Some(ready) = ready {
        let _ = ready.send(());
    }
    let capacity = Arc::new(Semaphore::new(64));
    let mut connections = tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => break,
            result = listener.connect() => {
                result?;
                let next = ipc::windows::listener_for_owner(endpoint, false, profile.operator_sid())?;
                let stream = std::mem::replace(&mut listener, next);
                let Ok(permit) = capacity.clone().try_acquire_owned() else { continue; };
                let gateway = gateway.clone();
                connections.spawn(async move {
                    let _permit = permit;
                    match tokio::time::timeout(Duration::from_secs(30), handle(stream, gateway)).await {
                        Ok(Ok(())) => {},
                        Ok(Err(error)) => tracing::debug!(%error, "machine gateway request failed"),
                        Err(_) => tracing::debug!("machine gateway request timed out"),
                    }
                });
            }
            Some(result) = connections.join_next(), if !connections.is_empty() => {
                if let Err(error) = result { tracing::debug!(%error, "machine gateway task failed"); }
            }
        }
    }
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    Ok(())
}

async fn handle(mut stream: NamedPipeServer, gateway: Arc<MachineGateway>) -> Result<()> {
    let bytes = ipc::frame::read(&mut stream).await?;
    // Identify the token of the last message, and revert before any await or dispatch.
    let peer = xrat_support::windows::named_pipe_client_sid(&stream)?;
    let result = match serde_json::from_slice::<GatewayRequest>(&bytes) {
        Ok(request) => gateway.dispatch(&peer, request).await,
        Err(error) => Err(error.into()),
    };
    let response = result.unwrap_or_else(|error| ipc::DaemonResponse {
        protocol_version: GATEWAY_VERSION,
        ok: false,
        code: ipc::DaemonResponseCode::InvalidState,
        message: error.to_string(),
        payload: None,
    });
    ipc::frame::write(&mut stream, &serde_json::to_vec(&response)?).await?;
    let _ = stream.read_u8().await?;
    Ok(())
}

pub async fn request(request: GatewayRequest) -> Result<ipc::DaemonResponse<GatewayPayload>> {
    let exchange = async {
        let mut stream = loop {
            match ipc::windows::connect(std::path::Path::new(ENDPOINT)) {
                Ok(stream) => break stream,
                Err(error) if error.raw_os_error() == Some(231) => {
                    tokio::time::sleep(Duration::from_millis(20)).await
                }
                Err(error) => return Err(error.into()),
            }
        };
        xrat_support::windows::require_system_owner(&stream)?;
        ipc::frame::write(&mut stream, &serde_json::to_vec(&request)?).await?;
        let response: ipc::DaemonResponse<GatewayPayload> =
            serde_json::from_slice(&ipc::frame::read(&mut stream).await?)?;
        if response.protocol_version != GATEWAY_VERSION {
            return Err(AppError::InvalidArgument(
                "unsupported machine gateway response version".into(),
            ));
        }
        stream.write_all(&[0]).await?;
        Ok(response)
    };
    tokio::time::timeout(Duration::from_secs(30), exchange)
        .await
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "machine gateway request timed out",
            )
        })?
}
