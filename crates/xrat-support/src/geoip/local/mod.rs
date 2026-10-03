use std::net::IpAddr;
use std::path::{Path, PathBuf};

use maxminddb::{Reader, geoip2};

use super::{GeoIpError, GeoIpLookup};

#[cfg(test)]
mod tests;

mod resolver;
pub use resolver::LocalMmdbLookup;
pub use resolver::lookup_asn_label;
pub use resolver::lookup_city_label;
pub use resolver::lookup_country_iso;
