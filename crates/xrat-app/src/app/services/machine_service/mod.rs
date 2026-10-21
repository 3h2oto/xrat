mod gateway;
mod protocol;

pub use gateway::MachineGateway;
pub use protocol::{GATEWAY_VERSION, GatewayPayload, GatewayRequest, GatewayRequestKind};

#[cfg(windows)]
mod profile;
#[cfg(windows)]
mod transport;
#[cfg(windows)]
pub use profile::MachineProfile;
#[cfg(windows)]
pub use transport::{request, serve};

#[cfg(windows)]
pub mod commands;
#[cfg(windows)]
mod host;
mod identity;
#[cfg(windows)]
pub mod installation;
#[cfg(any(windows, test))]
mod lifecycle;
#[cfg(windows)]
pub mod scm;
#[cfg(windows)]
pub use host::run_service;
#[cfg(any(windows, test))]
use identity::validate_operator;
pub use identity::{ENDPOINT, SYSTEM_SID};

#[cfg(all(test, windows))]
mod native_tests;
