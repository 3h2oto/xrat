use crate::app::config::{AppConfig, DatabaseBackend};
use crate::app::context::AppContext;
use crate::app::runtime_service::RuntimeService;
use crate::app::{AppError, Result};

pub(super) fn validate_config(config: &AppConfig) -> Result<()> {
    if config.paths.database.is_some()
        || config.paths.xray.is_some()
        || config.paths.v2ray.is_some()
        || config.paths.sing_box.is_some()
        || config.database.sqlite.path.is_some()
        || !matches!(config.database.backend, DatabaseBackend::Sqlite)
    {
        return Err(AppError::InvalidArgument(
            "machine service requires its protected SQLite database and core paths; remove database/path overrides from the service configuration".into(),
        ));
    }
    Ok(())
}

pub(super) async fn prepare_runtime(context: &mut AppContext) -> Result<()> {
    context.app_config.server.enabled = false;
    context.app_config.runtime.rotation.enabled = false;
    context.app_config.runtime.rotation.refresh_subscriptions = false;
    context.app_config.subscriptions.auto_refresh = false;
    let runtime = RuntimeService::new(context);
    let _ = runtime
        .reconcile_reattach_on_daemon_start(&uuid::Uuid::new_v4().to_string())
        .await?;
    runtime.disconnect().await?;
    runtime.cleanup_inactive_tun_ownership().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
