use tokio::sync::{mpsc, oneshot};

use crate::app::daemon::supervisor::SupervisorEvent;

mod ping_shutdown;
mod proxy;
mod runtime;

pub use ping_shutdown::{daemon_shutdown_response_via_supervisor, ping_response_via_supervisor};
pub use proxy::{
    proxy_start_response_via_supervisor, proxy_status_response_via_supervisor,
    proxy_stop_response_via_supervisor,
};
pub use runtime::{
    runtime_connect_response_via_supervisor, runtime_disconnect_response_via_supervisor,
    runtime_replace_response_via_supervisor, runtime_status_response_via_supervisor,
    runtime_tun_response_via_supervisor,
};

mod socket;
pub(crate) use socket::roundtrip;
