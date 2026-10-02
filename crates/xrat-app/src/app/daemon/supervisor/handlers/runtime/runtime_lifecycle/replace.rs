use super::*;
use crate::app::runtime_service::ReplaceRequest;

pub(super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<ConfigId>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    let active_session = context
        .db
        .get_running_runtime_session()
        .await
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "runtime_replace_session", %error, "failed to load optional supervisor state");
            None
        });
    if let Some(session) = &active_session {
        let started_reason = rotation_started_reason(trigger);
        if let Err(error) = context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("daemon"),
                Some(&state.instance_id),
                Some(started_reason),
                Some("proxy rotation replacement requested"),
                Some("daemon"),
            )
            .await
        {
            tracing::debug!(operation = "runtime_replace_started_metadata", session_id = session.id, %error, "failed to persist supervisor metadata");
        }
    }

    match RuntimeService::new(context)
        .replace(ReplaceRequest {
            trigger,
            candidate_id,
        })
        .await
    {
        Ok(result) => {
            state.last_trigger = Some(trigger);
            state.last_result = "replace_commit_success".to_string();
            state.last_candidate_config_id = Some(result.new_config_id);
            state.last_candidate_result = "replace_commit_success".to_string();
            state.cooldown_active = false;
            state.pending_health_recovery = false;
            state.consecutive_health_failures = 0;
            if state.rotation_enabled {
                state.next_timer_epoch_secs =
                    Some(now_epoch_seconds() + state.rotation_interval_secs);
            }
            if let Err(error) = context
                .db
                .update_runtime_session_transition_metadata(
                    result.new_session_id,
                    Some("daemon"),
                    Some(&state.instance_id),
                    Some("replace_commit_success"),
                    Some("daemon replace handoff completed"),
                    Some("daemon"),
                )
                .await
            {
                tracing::debug!(operation = "runtime_replace_committed_metadata", session_id = result.new_session_id, %error, "failed to persist supervisor metadata");
            }
            crate::app::events::record(
                &context.db,
                crate::app::events::LEVEL_INFO,
                crate::app::events::SOURCE_ROTATION,
                "proxy_rotated",
                format!(
                    "Rotated to config {} ({})",
                    result.new_config_id,
                    rotation_trigger_label(trigger)
                ),
                Some(result.new_config_id),
                Some(result.new_session_id),
                None,
            )
            .await;
            if respond_to
                .send(RuntimeReplaceResult::Ok(RuntimeReplacePayload {
                    trigger,
                    replaced: result.old_session_id.is_some(),
                    old_session_id: result.old_session_id,
                    new_config_id: result.new_config_id,
                    new_session_id: result.new_session_id,
                    new_pid: result.new_pid,
                }))
                .is_err()
            {
                tracing::debug!(
                    operation = "runtime_replace_response",
                    ?trigger,
                    ?candidate_id,
                    error = "response receiver dropped",
                    "supervisor response dropped"
                );
            }
        }
        Err(err) => {
            let message = err.to_string();
            let failure_reason = rotation_failure_reason(&message);
            state.last_trigger = Some(trigger);
            state.last_result = failure_reason.to_string();
            state.last_candidate_config_id = candidate_id;
            state.last_candidate_result = failure_reason.to_string();
            state.cooldown_active = trigger == RotationTrigger::HealthCheckFailed;
            state.pending_health_recovery = trigger == RotationTrigger::HealthCheckFailed;
            if state.rotation_enabled {
                state.next_timer_epoch_secs =
                    Some(now_epoch_seconds() + state.rotation_interval_secs);
            }
            if let Some(session) = &active_session
                && let Err(error) = context
                    .db
                    .update_runtime_session_transition_metadata(
                        session.id,
                        Some("daemon"),
                        Some(&state.instance_id),
                        Some(failure_reason),
                        Some(&message),
                        Some("daemon"),
                    )
                    .await
            {
                tracing::debug!(operation = "runtime_replace_failed_metadata", session_id = session.id, %error, "failed to persist supervisor metadata");
            }
            crate::app::events::record(
                &context.db,
                crate::app::events::LEVEL_WARN,
                crate::app::events::SOURCE_ROTATION,
                "rotation_failed",
                format!(
                    "Rotation failed ({}): {message}",
                    rotation_trigger_label(trigger)
                ),
                candidate_id,
                None,
                None,
            )
            .await;
            if respond_to
                .send(RuntimeReplaceResult::Err { message })
                .is_err()
            {
                tracing::debug!(
                    operation = "runtime_replace_error_response",
                    ?trigger,
                    ?candidate_id,
                    error = "response receiver dropped",
                    "supervisor response dropped"
                );
            }
        }
    }
}
