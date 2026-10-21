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

mod identity;
#[cfg(any(windows, test))]
use identity::validate_operator;
pub use identity::{ENDPOINT, SYSTEM_SID};
