mod control;
mod daemon;
mod factory;
mod local;
#[cfg(any(windows, test))]
mod machine;
#[cfg(any(windows, test))]
pub use machine::MachineRuntimeControl;

pub use control::{RuntimeConnectOutcome, RuntimeControl, RuntimeReplaceOutcome};
pub use daemon::DaemonRuntimeControl;
#[cfg(any(unix, windows))]
pub(crate) use factory::tui_uses_daemon;
pub use factory::{daemon_control, local_control, tui_control};
pub use local::LocalRuntimeControl;
