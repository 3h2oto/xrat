use crate::app::daemon::ipc::{DaemonResponse, DaemonResponseCode, PROTOCOL_VERSION};

mod daemon_ping;
mod proxy;
mod runtime;

pub use daemon_ping::{daemon_shutdown_response, ping_response};
pub use proxy::{
    proxy_control_error_response, proxy_control_response, proxy_status_error_response,
    proxy_status_response,
};
pub use runtime::{
    runtime_connect_error_response, runtime_connect_response, runtime_disconnect_error_response,
    runtime_disconnect_response, runtime_replace_error_response, runtime_replace_response,
    runtime_status_error_response, runtime_status_response,
};

mod builders;
use builders::*;
