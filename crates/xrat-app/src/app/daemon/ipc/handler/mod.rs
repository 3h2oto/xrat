use std::path::Path;

#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use crate::app::daemon::ipc::daemon_unreachable;
use crate::app::daemon::supervisor::SupervisorEvent;

#[cfg(unix)]
mod dispatch;
#[cfg(unix)]
mod io;

mod requests;
#[cfg(unix)]
pub use requests::serve_ping;
#[cfg(not(unix))]
pub use requests::serve_ping;
