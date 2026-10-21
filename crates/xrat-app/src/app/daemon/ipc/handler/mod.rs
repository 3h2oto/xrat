use std::path::Path;

#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::sync::mpsc;

#[cfg(unix)]
use crate::app::daemon::ipc::daemon_unreachable;
use crate::app::daemon::supervisor::SupervisorEvent;

#[cfg(any(unix, windows))]
mod dispatch;
#[cfg(unix)]
mod io;

mod requests;
pub use requests::serve_ping;

#[cfg(windows)]
mod windows;
