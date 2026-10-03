use super::*;

#[derive(Debug, thiserror::Error)]
pub enum GeoIpError {
    #[error("geoip HTTP request failed: {0}")]
    Http(#[from] crate::http::HttpError),
    #[error("geoip service returned status {status} for {ip}")]
    Status {
        ip: String,
        status: u16,
        body_preview: String,
    },
    #[error("geoip service response was malformed: {0}")]
    Parse(String),
    #[error("geoip rate limit exceeded; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("geoip backend not configured")]
    NotConfigured,
    #[error("invalid geoip settings: {0}")]
    InvalidSettings(String),
}

#[async_trait::async_trait]
pub trait GeoIpLookup: Send + Sync + Debug {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    fn backend_name(&self) -> &'static str;
}
