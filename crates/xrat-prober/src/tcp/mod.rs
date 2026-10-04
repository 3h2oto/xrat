mod check;
mod classify;
mod model;

use super::FailureKind;
pub use check::{tcp_check, tcp_check_with_ports};
pub use model::TcpResult;

#[cfg(test)]
mod tests;
