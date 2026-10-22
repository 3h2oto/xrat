mod catalog;
mod gateway;
mod probes;
mod protocol;
pub use probes::{
    ProbeProgressPage, ProbeRequest, ProbeResult, ProbeResultPage, ProbeSnapshot, ProbeStages,
    ProbeState,
};
#[cfg(any(windows, test))]
mod response;
#[cfg(windows)]
pub(crate) use response::encode_response;
#[cfg(any(windows, test))]
mod client;
pub use catalog::{CatalogPayload, CatalogRequest};
#[cfg(any(windows, test))]
pub use client::MachineClient;
#[cfg(test)]
pub(crate) use client::Transport;

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
mod host;
mod identity;
#[cfg(any(windows, test))]
mod lifecycle;
#[cfg(windows)]
pub mod scm;
#[cfg(windows)]
pub use host::run_service;
#[cfg(any(windows, test))]
pub(crate) use identity::validate_operator;
pub use identity::{ENDPOINT, SYSTEM_SID};

#[cfg(all(test, windows))]
mod native_tests;
