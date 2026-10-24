use super::{AppContext, DaemonOverview, RuntimeService, RuntimeStatusSnapshot, daemon};
use crate::app::daemon::ipc::TunStatePayload;

pub(super) struct DashboardRuntime {
    pub runtime: RuntimeStatusSnapshot,
    pub tun: TunStatePayload,
    pub daemon: DaemonOverview,
    pub config_path: String,
    pub test_stages: Vec<String>,
}

pub(super) async fn load(context: &AppContext) -> crate::app::Result<DashboardRuntime> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        let overview = client.overview().await?;
        return Ok(DashboardRuntime {
            runtime: overview.runtime,
            tun: overview.tun,
            daemon: DaemonOverview {
                running: overview.daemon_ready,
                rotation_enabled: overview.rotation_enabled,
                interval_secs: overview.rotation_interval_secs,
            },
            config_path: overview.config_path,
            test_stages: overview.test_stages,
        });
    }
    let runtime = RuntimeService::new(context).status().await?;
    let tun = crate::app::services::tun::capture_state(context, &runtime, &Default::default());
    Ok(DashboardRuntime {
        runtime,
        tun,
        daemon: daemon::load_daemon_info(context).await,
        config_path: context.runtime_paths.config_path.display().to_string(),
        test_stages: context.app_config.runtime.rotation.test_stages.clone(),
    })
}
