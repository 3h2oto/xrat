mod cache;
mod chain;
mod classify;
mod enrich;
mod fronting;
mod local;
mod rate_limit;
mod remote_ip_api;
mod remote_ipwhois;

use std::net::IpAddr;

use std::fmt::Debug;

pub use cache::CachedLookup;
pub use chain::ChainedLookup;
pub use classify::{classify_endpoint_location, is_classified_placeholder};
pub use enrich::{EndpointGeoMeta, GeoIpSource, address_host, enrich_address, resolve_address_ip};
pub use fronting::detect_fronting;
pub use local::{LocalMmdbLookup, lookup_asn_label, lookup_city_label, lookup_country_iso};
pub use rate_limit::RateLimitedLookup;
pub use remote_ip_api::RemoteIpApiLookup;
pub use remote_ipwhois::RemoteIpWhoisLookup;

mod lookup;
pub use lookup::GeoIpError;
pub use lookup::GeoIpLookup;
