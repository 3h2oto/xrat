use super::*;
use crate::app::services::runtime_transitions::RuntimeTransitionService;

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    let outcome = RuntimeTransitionService::new(context, &state.instance_id)
        .disconnect()
        .await;
    let operation = if outcome.result.is_ok() {
        "runtime_disconnect_response"
    } else {
        "runtime_disconnect_error_response"
    };
    let response = match outcome.result {
        Ok(result) => RuntimeDisconnectResult::Ok(RuntimeDisconnectPayload {
            stopped_session: result.stopped_session,
        }),
        Err(error) => RuntimeDisconnectResult::Err {
            message: error.to_string(),
        },
    };
    if respond_to.send(response).is_err() {
        tracing::debug!(
            operation,
            session_id = ?outcome.session_id,
            error = "response receiver dropped",
            "supervisor response dropped"
        );
    }
}
