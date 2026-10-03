use std::net::IpAddr;
use std::time::Duration;

use serde::Deserialize;

use super::{GeoIpError, GeoIpLookup};

#[cfg(test)]
mod tests;

mod resolver;
pub use resolver::RemoteIpWhoisLookup;
