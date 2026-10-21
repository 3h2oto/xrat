use super::*;

mod process;

mod recovery;
pub(super) use recovery::ProcessInspector;

#[cfg(windows)]
pub(super) fn verified_system_session(
    context: &AppContext,
    session: &RuntimeSessionRecord,
) -> bool {
    recovery::validate_reattach_session(context, session, &process::SystemProcessInspector)
}
