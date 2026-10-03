use super::super::*;
use super::stale::mark_session_stale;

pub(crate) async fn active_session_state(
    context: &AppContext,
    signals: &dyn xrat_support::signals::ProcessSignals,
) -> crate::app::Result<ActiveSessionState> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveSessionState::None);
    };
    if runtime_session_is_alive(&session, signals) {
        return Ok(ActiveSessionState::Running(session));
    }
    mark_session_stale(context, &session).await?;
    Ok(ActiveSessionState::Stale(session))
}

pub(crate) fn runtime_session_is_alive(
    session: &RuntimeSessionRecord,
    signals: &dyn xrat_support::signals::ProcessSignals,
) -> bool {
    session
        .process_id
        .map(|pid| signals.is_running(pid))
        .unwrap_or(false)
}
