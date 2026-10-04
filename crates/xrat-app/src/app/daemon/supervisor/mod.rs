use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc;
use tokio::time::{self, Duration};

mod handlers;
mod types;

pub use types::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorEvent,
    SupervisorState, channel,
};

use crate::app::context::AppContext;
use crate::app::events;
use crate::app::runtime_service::{ConnectRequest, RuntimeService};
use crate::app::subscription_refresh;
use xrat_model::ConfigId;
use xrat_support::time::now_epoch_seconds;

mod state;
pub use state::run;
