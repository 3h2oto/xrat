use crate::app::context::AppContext;
use crate::app::daemon::ipc::{
    DaemonShutdownPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger,
    RuntimeDisconnectPayload, RuntimeReplacePayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeDisconnectResult,
    RuntimeReplaceResult, SupervisorState,
};
use tokio::sync::oneshot;
use xrat_model::ConfigId;
use xrat_support::time::now_epoch_seconds;

mod disconnect;
mod proxy;
mod replace;

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    disconnect::handle_runtime_disconnect(state, context, respond_to).await;
}

pub(super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<ConfigId>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    replace::handle_runtime_replace(state, context, trigger, candidate_id, respond_to).await;
}

pub(super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    proxy::handle_proxy_start(state, context, respond_to).await;
}

pub(super) async fn handle_proxy_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyStatusResult>,
) {
    proxy::handle_proxy_status(state, context, respond_to).await;
}

pub(super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    proxy::handle_proxy_stop(state, context, respond_to).await;
}

pub(super) async fn handle_daemon_shutdown(
    context: &AppContext,
    respond_to: oneshot::Sender<DaemonShutdownResult>,
) {
    let runtime_disconnected =
        crate::app::services::runtime_transitions::RuntimeTransitionService::shutdown(context)
            .await;
    if respond_to
        .send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
            daemon_ready: false,
            runtime_disconnected,
        }))
        .is_err()
    {
        tracing::debug!(
            operation = "daemon_shutdown_response",
            error = "response receiver dropped",
            "supervisor response dropped"
        );
    }
}
