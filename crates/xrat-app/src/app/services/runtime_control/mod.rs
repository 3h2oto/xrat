mod control;
mod daemon;
mod factory;
mod local;

pub use control::{RuntimeConnectOutcome, RuntimeControl, RuntimeReplaceOutcome};
pub use daemon::DaemonRuntimeControl;
#[cfg(unix)]
pub(crate) use factory::tui_uses_daemon;
pub use factory::{daemon_control, local_control, tui_control};
pub use local::LocalRuntimeControl;
