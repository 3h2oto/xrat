use std::sync::Arc;

use crate::app::context::AppContext;
use crate::app::daemon::ipc::{self};

use super::control::RuntimeControl;
use super::daemon::DaemonRuntimeControl;
use super::local::LocalRuntimeControl;

/// Build the control implementation for a context.
///
/// The daemon path is used whenever a runtime directory is configured, matching
/// current CLI behavior of talking to the daemon socket. Callers that know they
/// run inside the daemon (or standalone TUI) should build
/// [`LocalRuntimeControl`] directly.
pub fn daemon_control(context: &AppContext) -> Arc<dyn RuntimeControl> {
    Arc::new(DaemonRuntimeControl::new(ipc::default_socket_path(
        &context.runtime_paths.runtime_dir,
    )))
}

/// Build an in-process control implementation.
pub fn local_control(context: &AppContext) -> Arc<dyn RuntimeControl> {
    Arc::new(LocalRuntimeControl::new(context.clone()))
}
