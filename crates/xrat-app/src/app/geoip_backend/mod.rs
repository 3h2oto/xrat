use std::sync::Arc;

use crate::app::config::{AppConfig, GeoIpBackend};
use crate::app::context::RuntimePaths;
use crate::app::paths::mmdb;

use xrat_support::geoip::{
    CachedLookup, ChainedLookup, GeoIpError, GeoIpLookup, LocalMmdbLookup, RateLimitedLookup,
    RemoteIpApiLookup, RemoteIpWhoisLookup,
};

mod validation;

#[cfg(test)]
mod tests;

mod backend;
pub(crate) use backend::build_lookup_chain;
