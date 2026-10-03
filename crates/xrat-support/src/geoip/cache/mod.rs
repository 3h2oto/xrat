use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::{GeoIpError, GeoIpLookup};

#[cfg(test)]
mod tests;

mod storage;
pub use storage::CachedLookup;
