use crate::app::services::{
    machine_service::{GatewayPayload, GatewayRequestKind, MachineClient},
    tun_control,
};
use crate::app::{AppError, Result, commands::output};
use crate::cli::{TunAction, TunArgs};
pub(super) async fn run(client: &MachineClient, args: &TunArgs) -> Result<()> {
    let enabled = matches!(args.action, TunAction::Enable(_));
    let (state, json) = match &args.action {
        TunAction::Status(args) => (client.overview().await?.tun, args.json),
        TunAction::Enable(args) | TunAction::Disable(args) => {
            if args.mode.is_some() {
                return Err(AppError::InvalidArgument("Machine split-tunnel policy requires administrator maintenance; configure its mode before enabling".into()));
            }
            let GatewayPayload::Tun(state) = client
                .call(GatewayRequestKind::Tun {
                    enabled: Some(enabled),
                })
                .await?
            else {
                return Err(super::entry::unexpected());
            };
            (state, args.json)
        }
        _ => return Err(AppError::InvalidArgument(
            "Machine TUN setup and split-tunnel policy require explicit administrator maintenance"
                .into(),
        )),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&state)?);
    } else {
        println!(
            "{}",
            output::success(tun_control::message(&state), output::color_enabled())
        );
    }
    Ok(())
}
