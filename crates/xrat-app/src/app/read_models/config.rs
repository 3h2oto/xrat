use xrat_db::record::ConfigWithLatestTest;

/// Interface-neutral summary of a stored config.
///
/// CLI tables/JSON, HTTP DTOs, and TUI rows all derive from this model so a new
/// config field only needs mapping in one place.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigSummary {
    pub id: i64,
    pub r#ref: String,
    pub name: Option<String>,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub network: String,
    pub tls: Option<String>,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_deleted: bool,
    pub subscription_id: Option<i64>,
    pub latest_test: Option<LatestTestSummary>,
}

/// Interface-neutral latest connection-test facts for a config.
#[derive(Clone, Debug, PartialEq)]
pub struct LatestTestSummary {
    pub icmp_ok: Option<bool>,
    pub icmp_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
    pub tested_at: Option<String>,
}

/// Interface-neutral endpoint geolocation facts attached to a config row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EndpointLocation {
    pub location: Option<String>,
    pub country: Option<String>,
    pub asn: Option<String>,
    pub geoip_source: Option<String>,
    pub fronting: Option<String>,
}

/// Interface-neutral detail view of a stored config.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigDetail {
    pub summary: ConfigSummary,
    pub subscription_id: Option<i64>,
    pub dedup_key: String,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub deleted_at: Option<String>,
    pub imported_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl ConfigDetail {
    pub fn from_joined(row: &ConfigWithLatestTest) -> Self {
        Self {
            summary: ConfigSummary::from_joined(row),
            subscription_id: row.config.subscription_id,
            dedup_key: row.config.dedup_key.clone(),
            sni: row.config.sni.clone(),
            host: row.config.host.clone(),
            path: row.config.path.clone(),
            deleted_at: row.config.deleted_at.clone(),
            imported_at: row.config.imported_at.clone(),
            created_at: row.config.created_at.clone(),
            updated_at: row.config.updated_at.clone(),
        }
    }
}

impl LatestTestSummary {
    /// Build a summary from a joined config row, or `None` when the config has
    /// never been tested.
    pub fn from_joined(row: &ConfigWithLatestTest) -> Option<Self> {
        row.test_id?;
        Some(Self {
            icmp_ok: row.icmp_ok,
            icmp_ms: row.icmp_ms,
            tcp_ok: row.tcp_ok,
            tcp_ms: row.tcp_ms,
            real_delay_ok: row.real_delay_ok,
            real_delay_ms: row.real_delay_ms,
            download_mbps: row.download_mbps,
            upload_mbps: row.upload_mbps,
            connect_ms: row.connect_ms,
            ttfb_ms: row.ttfb_ms,
            http_status: row.http_status,
            failure_kind: row.failure_kind.clone(),
            failure_reason: row.failure_reason.clone(),
            tested_at: row.tested_at.clone(),
        })
    }
}

impl ConfigSummary {
    /// Map a joined config/test row into an interface-neutral summary.
    pub fn from_joined(row: &ConfigWithLatestTest) -> Self {
        Self {
            id: row.config.id,
            r#ref: row.config.r#ref.clone(),
            name: row.config.name.clone(),
            protocol: row.config.protocol.clone(),
            address: row.config.address.clone(),
            port: row.config.port,
            network: row.config.network.clone(),
            tls: row.config.tls.clone(),
            is_active: row.config.is_active,
            is_enabled: row.config.is_enabled,
            is_deleted: row.config.is_deleted,
            subscription_id: row.config.subscription_id,
            latest_test: LatestTestSummary::from_joined(row),
        }
    }

    /// Endpoint facts from the joined row.
    pub fn endpoint_location(row: &ConfigWithLatestTest) -> EndpointLocation {
        EndpointLocation {
            location: row.dial_endpoint_location.clone(),
            country: row.dial_endpoint_country.clone(),
            asn: row.dial_endpoint_asn.clone(),
            geoip_source: row.dial_endpoint_geoip_source.clone(),
            fronting: row.dial_endpoint_fronting.clone(),
        }
    }
}
