use crate::app::context::AppContext;
use crate::app::daemon::ipc::{
    DaemonShutdownPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger,
    RuntimeDisconnectPayload, RuntimeReplacePayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeDisconnectResult,
    RuntimeReplaceResult, SupervisorState,
};
use tokio::sync::oneshot;
use xrat_model::ConfigId;
use xrat_support::time::now_epoch_seconds;

mod disconnect;
mod proxy;
mod replace;

mod lifecycle;
pub(super) use lifecycle::handle_daemon_shutdown;
pub(super) use lifecycle::handle_proxy_start;
pub(super) use lifecycle::handle_proxy_status;
pub(super) use lifecycle::handle_proxy_stop;
pub(super) use lifecycle::handle_runtime_disconnect;
pub(super) use lifecycle::handle_runtime_replace;
