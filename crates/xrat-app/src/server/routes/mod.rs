pub(crate) mod b64;
pub(crate) mod configs;
pub(crate) mod health;
pub(crate) mod json;
pub(crate) mod pac;

use axum::Router;
use axum::routing::get;

use crate::server::ServerState;

mod router;
pub use router::router;
