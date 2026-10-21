use super::{RuntimeConnectOutcome, RuntimeControl, RuntimeReplaceOutcome};
use crate::app::daemon::ipc::RotationTrigger;
use crate::app::services::machine_service::{
    self, GATEWAY_VERSION, GatewayPayload, GatewayRequest, GatewayRequestKind,
};
use crate::app::{AppError, Result};
use xrat_model::ConfigId;

/// Runtime adapter for the protected machine service; never falls back to local execution.
pub struct MachineRuntimeControl;

async fn call(request: GatewayRequestKind) -> Result<GatewayPayload> {
    let response = machine_service::request(GatewayRequest {
        protocol_version: GATEWAY_VERSION,
        request,
    })
    .await?;
    if !response.ok {
        return Err(AppError::InvalidArgument(response.message));
    }
    response
        .payload
        .ok_or_else(|| AppError::InvalidArgument("machine gateway response missing payload".into()))
}

#[async_trait::async_trait]
impl RuntimeControl for MachineRuntimeControl {
    async fn connect(&self, config_id: ConfigId) -> Result<RuntimeConnectOutcome> {
        let GatewayPayload::Connect(payload) =
            call(GatewayRequestKind::Connect { config_id }).await?
        else {
            return Err(AppError::InvalidArgument(
                "unexpected machine gateway connect response".into(),
            ));
        };
        Ok(RuntimeConnectOutcome {
            config_id: payload.config_id,
            session_id: payload.session_id,
            pid: payload.pid,
        })
    }

    async fn disconnect(&self) -> Result<bool> {
        let GatewayPayload::Disconnect(payload) = call(GatewayRequestKind::Disconnect {}).await?
        else {
            return Err(AppError::InvalidArgument(
                "unexpected machine gateway disconnect response".into(),
            ));
        };
        Ok(payload.stopped_session)
    }

    async fn replace(
        &self,
        trigger: RotationTrigger,
        candidate_id: Option<ConfigId>,
    ) -> Result<RuntimeReplaceOutcome> {
        if trigger != RotationTrigger::Manual {
            return Err(AppError::InvalidArgument(
                "machine clients may request only manual replacement".into(),
            ));
        }
        let GatewayPayload::Replace(payload) =
            call(GatewayRequestKind::Replace { candidate_id }).await?
        else {
            return Err(AppError::InvalidArgument(
                "unexpected machine gateway replace response".into(),
            ));
        };
        Ok(RuntimeReplaceOutcome {
            trigger: payload.trigger,
            replaced: payload.replaced,
            old_session_id: payload.old_session_id,
            new_config_id: payload.new_config_id,
            new_session_id: payload.new_session_id,
            new_pid: payload.new_pid,
        })
    }
}
