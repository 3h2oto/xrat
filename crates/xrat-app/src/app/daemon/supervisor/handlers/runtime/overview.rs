use crate::app::context::AppContext;
use crate::app::daemon::supervisor::SupervisorState;
use crate::app::read_models::RuntimeOverview;
use crate::app::runtime_service::RuntimeService;
use tokio::sync::oneshot;

pub(crate) async fn handle_runtime_overview(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<Result<Box<RuntimeOverview>, String>>,
) {
    let result = async {
        let runtime = RuntimeService::new(context).status().await?;
        let tun = crate::app::services::tun::capture_state(
            context,
            &runtime,
            &xrat_support::readiness::RuntimeProcessPorts::default(),
        );
        Ok::<_, crate::app::AppError>(Box::new(RuntimeOverview {
            runtime,
            tun,
            daemon_ready: state.ready,
            rotation_enabled: state.rotation_enabled,
            rotation_interval_secs: state.rotation_interval_secs,
            test_stages: context.app_config.runtime.rotation.test_stages.clone(),
            config_path: context.runtime_paths.config_path.display().to_string(),
        }))
    }
    .await
    .map_err(|error| error.to_string());
    if respond_to.send(result).is_err() {
        tracing::debug!("runtime overview receiver dropped");
    }
}

#[cfg(test)]
mod tests;
