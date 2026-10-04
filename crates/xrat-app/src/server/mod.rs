mod auth;
mod error;
mod response;
mod routes;
mod state;

#[cfg(test)]
mod tests;

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use tokio::net::TcpListener;

use crate::app::config::{RoutingSettings, ServerSettings};
use crate::app::events;
use xrat_db::Database;

pub use error::{ServerError, ServerResult};
pub use routes::pac::proxy_pac;
pub use state::ServerState;

mod listener;
pub use listener::build_router;
pub use listener::build_router_from_context;
pub use listener::parse_bind_addr_public;
pub use listener::serve;
pub use listener::serve_with_shutdown;
pub use listener::serve_with_signal;
