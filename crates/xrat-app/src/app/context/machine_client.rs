use std::path::Path;

use super::{AppContext, RuntimePaths};
use crate::app::services::machine_service::MachineClient;
use crate::app::{AppError, Result};
use xrat_db::{Database, DatabaseConnectionConfig};

pub(super) async fn compose(client: MachineClient, root: &Path) -> Result<AppContext> {
    let overview = client.overview().await?;
    if !overview.daemon_ready {
        return Err(AppError::InvalidArgument(
            "machine service is not ready".into(),
        ));
    }
    let paths = crate::app::app_paths::ensure_layout_at(root)?;
    let app_config = crate::app::config::load(&paths.config_path)?;
    let database_config = DatabaseConnectionConfig::Sqlite {
        path: paths.database_path.clone(),
    };
    let db = Database::connect(&database_config).await?;
    Ok(AppContext {
        db,
        app_config,
        runtime_paths: RuntimePaths {
            database_label: database_config.label(),
            database_config,
            database_path: paths.database_path,
            config_path: paths.config_path,
            runtime_dir: paths.root_dir.join("runtime"),
            xray_path: paths.root_dir.join("cores/xray.exe"),
            v2ray_path: paths.root_dir.join("cores/v2ray.exe"),
            sing_box_path: paths.root_dir.join("cores/sing-box.exe"),
            root_dir: paths.root_dir,
        },
        machine_client: Some(client),
    })
}

#[cfg(windows)]
pub(crate) fn require_ordinary_user() -> Result<()> {
    validate_identity(
        &xrat_support::windows::current_user_sid()?,
        xrat_support::windows::is_current_process_elevated()?,
    )
}

pub(super) fn validate_identity(sid: &str, elevated: bool) -> Result<()> {
    crate::app::services::machine_service::validate_operator(sid)?;
    if elevated {
        return Err(AppError::InvalidArgument(
            "machine TUI/Web frontends require an ordinary user token; run without elevation"
                .into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
