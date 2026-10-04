use std::io::ErrorKind;
use std::path::{Path, PathBuf};

mod client;
mod handler;
mod responses;
mod transport;
mod types;

pub use client::{
    daemon_shutdown_daemon, ping_daemon, proxy_start_daemon, proxy_status_daemon,
    proxy_stop_daemon, runtime_connect_daemon, runtime_disconnect_daemon, runtime_replace_daemon,
    runtime_status_daemon,
};
pub use handler::serve_ping;
pub use responses::{
    daemon_shutdown_response, ping_response, proxy_control_error_response, proxy_control_response,
    proxy_status_error_response, proxy_status_response, runtime_connect_error_response,
    runtime_connect_response, runtime_disconnect_error_response, runtime_disconnect_response,
    runtime_replace_error_response, runtime_replace_response, runtime_status_error_response,
    runtime_status_response,
};
pub use types::*;

#[cfg(all(test, unix))]
mod tests;

mod requests;
pub use requests::PROTOCOL_VERSION;
pub use requests::daemon_unreachable;
pub use requests::default_socket_path;
