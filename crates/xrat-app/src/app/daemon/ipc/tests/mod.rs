use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;

use crate::app::daemon::ipc::{
    DaemonResponse, DaemonShutdownPayload, PingPayload, RuntimeReplacePayload,
    daemon_shutdown_daemon, ping_daemon,
};
use crate::app::daemon::supervisor::{DaemonShutdownResult, RuntimeReplaceResult, SupervisorEvent};

mod lifecycle_cases;
mod replace_cases;
mod wire_cases;

mod fixtures;
use fixtures::*;
