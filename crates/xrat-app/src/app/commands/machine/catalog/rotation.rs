use crate::app::Result;
use crate::app::commands::{output, rotate};
use crate::app::daemon::ipc;
use crate::app::services::{
    ConfigListRequest,
    machine_service::{GatewayPayload, GatewayRequestKind, MachineClient},
};
use crate::cli::{RotateAction, RotateArgs};
use xrat_model::{ConfigId, ConfigRef};

pub(super) async fn run(client: &MachineClient, args: &RotateArgs) -> Result<()> {
    match &args.action {
        RotateAction::Enable(_) | RotateAction::Disable(_) => {
            let enabled = matches!(args.action, RotateAction::Enable(_));
            let request = if enabled {
                GatewayRequestKind::RotationStart {}
            } else {
                GatewayRequestKind::RotationStop {}
            };
            let GatewayPayload::RotationControl(payload) = client.call(request).await? else {
                return Err(super::entry::unexpected());
            };
            println!(
                "{}",
                output::success(
                    format!(
                        "Proxy rotation: {}.",
                        output::bool_label(payload.rotation_enabled)
                    ),
                    output::color_enabled()
                )
            );
            println!(
                "{}",
                output::notice(
                    "Machine service startup always disables rotation; enable it explicitly after restart.",
                    output::color_enabled()
                )
            );
        }
        RotateAction::Status(args) => {
            let GatewayPayload::RotationStatus(payload) =
                client.call(GatewayRequestKind::RotationStatus {}).await?
            else {
                return Err(super::entry::unexpected());
            };
            let active = reference(client, payload.active_config_id).await?;
            let candidate = reference(client, payload.last_candidate_config_id).await?;
            rotate::print_status(payload, active, candidate, args.json)?;
        }
        RotateAction::Now(args) => {
            if args.refresh {
                let summary = client.refresh_subscriptions(&[]).await?;
                crate::app::commands::update::print_summary(&summary);
            }
            let candidate_id = match args.config_id.as_deref() {
                Some(reference) => Some(client.resolve_config(&ConfigRef::from(reference)).await?),
                None => None,
            };
            let payload = match client
                .call(GatewayRequestKind::Replace { candidate_id })
                .await
            {
                Ok(GatewayPayload::Replace(payload)) => payload,
                Ok(_) => return Err(super::entry::unexpected()),
                Err(error) if rotate::is_no_running_runtime_session_error(&error.to_string()) => {
                    initial_connection(client, candidate_id).await?
                }
                Err(error) => return Err(error),
            };
            let reference = client
                .config_reference(payload.new_config_id)
                .await?
                .unwrap_or_else(|| "-".into());
            rotate::print_rotation_result(payload, reference);
        }
    }
    Ok(())
}

async fn reference(client: &MachineClient, id: Option<ConfigId>) -> Result<Option<String>> {
    match id {
        Some(id) => client.config_reference(id).await,
        None => Ok(None),
    }
}

async fn initial_connection(
    client: &MachineClient,
    candidate: Option<ConfigId>,
) -> Result<ipc::RuntimeReplacePayload> {
    let id = match candidate {
        Some(id) => id,
        None => {
            let configs = client
                .list(&ConfigListRequest {
                    only_enabled: true,
                    ..Default::default()
                })
                .await?;
            crate::app::services::rotation::initial_candidate(
                configs.items.into_iter().map(|row| row.summary.id),
            )?
        }
    };
    let GatewayPayload::Connect(payload) = client
        .call(GatewayRequestKind::Connect { config_id: id })
        .await?
    else {
        return Err(super::entry::unexpected());
    };
    Ok(ipc::RuntimeReplacePayload {
        trigger: ipc::RotationTrigger::Manual,
        replaced: false,
        old_session_id: None,
        new_config_id: payload.config_id,
        new_session_id: payload.session_id,
        new_pid: payload.pid,
    })
}
