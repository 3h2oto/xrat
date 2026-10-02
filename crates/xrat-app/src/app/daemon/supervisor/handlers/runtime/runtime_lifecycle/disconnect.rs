use super::*;

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    let active_session_id = context
        .db
        .get_running_runtime_session()
        .await
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "runtime_disconnect_session", %error, "failed to load optional supervisor state");
            None
        })
        .map(|session| session.id);
    match RuntimeService::new(context).disconnect().await {
        Ok(result) => {
            if result.stopped_session
                && let Some(session_id) = active_session_id
                && let Err(error) = context
                    .db
                    .update_runtime_session_transition_metadata(
                        session_id,
                        Some("daemon"),
                        Some(&state.instance_id),
                        Some("manual_disconnect"),
                        Some("daemon runtime disconnect request succeeded"),
                        Some("daemon"),
                    )
                    .await
            {
                tracing::debug!(operation = "runtime_disconnect_metadata", session_id = session_id, %error, "failed to persist supervisor metadata");
            }
            if result.stopped_session {
                crate::app::events::record(
                    &context.db,
                    crate::app::events::LEVEL_INFO,
                    crate::app::events::SOURCE_RUNTIME,
                    "disconnect",
                    "Disconnected managed runtime",
                    None,
                    active_session_id,
                    None,
                )
                .await;
            }
            if respond_to
                .send(RuntimeDisconnectResult::Ok(RuntimeDisconnectPayload {
                    stopped_session: result.stopped_session,
                }))
                .is_err()
            {
                tracing::debug!(operation = "runtime_disconnect_response", session_id = ?active_session_id, error = "response receiver dropped", "supervisor response dropped");
            }
        }
        Err(err) => {
            if respond_to
                .send(RuntimeDisconnectResult::Err {
                    message: err.to_string(),
                })
                .is_err()
            {
                tracing::debug!(operation = "runtime_disconnect_error_response", session_id = ?active_session_id, error = "response receiver dropped", "supervisor response dropped");
            }
        }
    }
}
