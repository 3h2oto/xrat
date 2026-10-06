use crate::app::context::AppContext;
use crate::app::daemon::ipc::{self, TunStatePayload};
use crate::app::runtime_service::RuntimeService;
use crate::app::{AppError, Result};

pub(crate) async fn apply(
    context: &mut AppContext,
    enabled: Option<bool>,
    standalone_owner: bool,
) -> Result<TunStatePayload> {
    #[cfg(unix)]
    {
        let socket = ipc::default_socket_path(&context.runtime_paths.runtime_dir);
        if super::runtime_control::tui_uses_daemon(&socket).await? {
            let ping = ipc::ping_daemon(&socket).await?;
            if !ping.payload.is_some_and(|payload| payload.live_tun) {
                return Err(AppError::InvalidArgument(
                    "The running daemon predates live TUN toggles. Restart it once after upgrading, then retry. The current connection was kept.".into(),
                ));
            }
            let response = ipc::runtime_tun_daemon(
                &socket,
                enabled,
                context.runtime_paths.config_path.clone(),
            )
            .await?;
            if !response.ok {
                return Err(AppError::InvalidArgument(response.message));
            }
            let payload = response
                .payload
                .ok_or_else(|| AppError::InvalidArgument("Daemon returned no TUN state".into()))?;
            reload(context)?;
            return Ok(payload);
        }
    }
    if !standalone_owner && RuntimeService::new(context).status().await?.pid_running {
        return Err(AppError::InvalidArgument(
            "The active runtime belongs to a standalone TUI. Press U there to toggle TUN, or use a daemon for CLI control. The current connection was kept.".into(),
        ));
    }
    super::tun::apply(context, enabled).await
}

pub(crate) fn reload(context: &mut AppContext) -> Result<()> {
    let contents = std::fs::read_to_string(&context.runtime_paths.config_path)?;
    let config: crate::app::config::AppConfig = toml::from_str(&contents)?;
    context.app_config.runtime = config.runtime;
    context.app_config.dns = config.dns;
    context.app_config.routing = config.routing;
    for (configured, binary) in [
        (&config.paths.xray, &mut context.runtime_paths.xray_path),
        (
            &config.paths.sing_box,
            &mut context.runtime_paths.sing_box_path,
        ),
        (&config.paths.v2ray, &mut context.runtime_paths.v2ray_path),
    ] {
        if let Some(path) = configured {
            *binary =
                crate::app::config::resolve_config_path(&context.runtime_paths.config_path, path);
        }
    }
    Ok(())
}

pub(crate) fn message(state: &TunStatePayload) -> String {
    let mode = if state.enabled { "enabled" } else { "disabled" };
    if let Some(config) = &state.active_config_ref {
        if state.active {
            format!(
                "TUN {mode} on config {config} via {} ({}).",
                state.engine,
                state.interface.as_deref().unwrap_or("unknown interface")
            )
        } else {
            format!("TUN {mode}; config {config} is connected in proxy mode.")
        }
    } else if state.enabled {
        "TUN enabled; it will apply on the next connection. Run `xrat tun status` to check privileges.".into()
    } else {
        "TUN disabled; future connections use proxy mode.".into()
    }
}
