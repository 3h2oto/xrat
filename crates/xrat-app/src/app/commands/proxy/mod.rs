mod desktop;
mod endpoints;
mod pac;
mod shell;

use crate::app::context::AppContext;
pub(crate) use crate::app::services::proxy_pac::ActiveEndpoints;
use crate::cli::{ProxyAction, ProxyArgs, ProxyPacAction};

mod dispatch;
pub(crate) use dispatch::http_proxy_url;
pub(crate) use dispatch::loopback_host;
pub(crate) use dispatch::resolve_active_endpoints;
pub use dispatch::run;
pub(crate) use dispatch::socks_proxy_url;
