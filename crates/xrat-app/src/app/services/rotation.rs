use crate::app::context::AppContext;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationTrigger {
    Manual,
    Timer,
    HealthCheckFailed,
}

use crate::app::runtime_service::{ReplaceRequest, ReplaceResult, RuntimeService};

#[derive(Debug, PartialEq, Eq)]
pub struct RotationFailure {
    pub message: String,
    pub reason: RotationFailureReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationFailureReason {
    NoCandidate,
    CandidateFailed,
}
impl RotationFailureReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoCandidate => "rotation_no_candidate",
            Self::CandidateFailed => "rotation_candidate_failed",
        }
    }
}

pub struct RotationService<'a> {
    context: &'a AppContext,
    instance_id: &'a str,
}
impl<'a> RotationService<'a> {
    pub fn new(context: &'a AppContext, instance_id: &'a str) -> Self {
        Self {
            context,
            instance_id,
        }
    }
    pub async fn replace(&self, request: ReplaceRequest) -> Result<ReplaceResult, RotationFailure> {
        let ReplaceRequest {
            trigger,
            candidate_id,
        } = request;
        let active_session = self.context
        .db
        .get_running_runtime_session()
        .await
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "runtime_replace_session", %error, "failed to load optional supervisor state");
            None
        });
        if let Some(session) = &active_session {
            let started_reason = rotation_started_reason(trigger);
            if let Err(error) = self
                .context
                .db
                .update_runtime_session_transition_metadata(
                    session.id,
                    Some("daemon"),
                    Some(self.instance_id),
                    Some(started_reason),
                    Some("proxy rotation replacement requested"),
                    Some("daemon"),
                )
                .await
            {
                tracing::debug!(operation = "runtime_replace_started_metadata", session_id = session.id, %error, "failed to persist supervisor metadata");
            }
        }

        match RuntimeService::new(self.context)
            .replace(ReplaceRequest {
                trigger,
                candidate_id,
            })
            .await
        {
            Ok(result) => {
                if let Err(error) = self
                    .context
                    .db
                    .update_runtime_session_transition_metadata(
                        result.new_session_id,
                        Some("daemon"),
                        Some(self.instance_id),
                        Some("replace_commit_success"),
                        Some("daemon replace handoff completed"),
                        Some("daemon"),
                    )
                    .await
                {
                    tracing::debug!(operation = "runtime_replace_committed_metadata", session_id = result.new_session_id, %error, "failed to persist supervisor metadata");
                }
                crate::app::events::record(
                    &self.context.db,
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
                Ok(result)
            }
            Err(err) => {
                let message = err.to_string();
                let failure_reason = rotation_failure_reason(&message);
                if let Some(session) = &active_session
                    && let Err(error) = self
                        .context
                        .db
                        .update_runtime_session_transition_metadata(
                            session.id,
                            Some("daemon"),
                            Some(self.instance_id),
                            Some(failure_reason.as_str()),
                            Some(&message),
                            Some("daemon"),
                        )
                        .await
                {
                    tracing::debug!(operation = "runtime_replace_failed_metadata", session_id = session.id, %error, "failed to persist supervisor metadata");
                }
                crate::app::events::record(
                    &self.context.db,
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
                Err(RotationFailure {
                    message,
                    reason: failure_reason,
                })
            }
        }
    }
    pub async fn enabled(&self, interval_secs: u64) {
        crate::app::events::record(
            &self.context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_ROTATION,
            "rotation_enabled",
            format!("Auto-rotation enabled (interval {}s)", interval_secs),
            None,
            None,
            None,
        )
        .await;
    }
    pub async fn disabled(&self) {
        crate::app::events::record(
            &self.context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_ROTATION,
            "rotation_disabled",
            "Auto-rotation disabled",
            None,
            None,
            None,
        )
        .await;
    }
}
fn rotation_started_reason(trigger: RotationTrigger) -> &'static str {
    match trigger {
        RotationTrigger::Manual => "rotation_manual_started",
        RotationTrigger::Timer => "rotation_timer_started",
        RotationTrigger::HealthCheckFailed => "rotation_health_started",
    }
}

fn rotation_trigger_label(trigger: RotationTrigger) -> &'static str {
    match trigger {
        RotationTrigger::Manual => "manual",
        RotationTrigger::Timer => "timer",
        RotationTrigger::HealthCheckFailed => "health check failed",
    }
}

fn rotation_failure_reason(message: &str) -> RotationFailureReason {
    if message.contains("no eligible replacement candidate") {
        RotationFailureReason::NoCandidate
    } else {
        RotationFailureReason::CandidateFailed
    }
}

#[cfg(test)]
#[path = "rotation_tests.rs"]
mod tests;
