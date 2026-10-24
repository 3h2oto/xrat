use super::super::*;

pub(crate) async fn stop_active_session(
    context: &AppContext,
    signals: &dyn xrat_support::signals::ProcessSignals,
) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };
    stop_session(context, &session, signals).await?;
    context
        .db
        .update_runtime_session_transition_metadata(
            session.id,
            Some("cli"),
            None,
            Some("manual_disconnect"),
            Some("runtime disconnect request succeeded"),
            Some("cli"),
        )
        .await?;
    context.db.clear_active_config().await?;
    Ok(true)
}

pub(crate) async fn stop_session(
    context: &AppContext,
    session: &RuntimeSessionRecord,
    signals: &dyn xrat_support::signals::ProcessSignals,
) -> crate::app::Result<()> {
    #[cfg(windows)]
    if xrat_support::windows::current_user_sid()?.as_str() == "S-1-5-18"
        && session
            .process_id
            .is_some_and(|pid| signals.is_running(pid))
        && !crate::app::runtime_service::reattach::verified_system_session(context, session)
    {
        return Err(AppError::InvalidArgument(
            "refusing privileged termination of an unverified persisted runtime process; reconcile stale session state before retrying".into(),
        ));
    }
    context
        .db
        .update_runtime_session_state(
            session.id,
            RuntimeSessionStatus::Stopping,
            None,
            None,
            None,
            None,
        )
        .await?;

    if let Some(pid) = session.process_id {
        let outcome = xray_runtime::terminate_process_gracefully_with_signals(
            pid,
            SHUTDOWN_TIMEOUT,
            signals,
        )?;
        tracing::info!(
            session_id = session.id,
            pid,
            outcome = ?outcome,
            "runtime process termination completed"
        );
    } else {
        tracing::warn!(
            session_id = session.id,
            "runtime session has no saved process id"
        );
    }

    context
        .db
        .mark_runtime_session_stopped(session.id, Some(&now_string()))
        .await?;
    Ok(())
}
