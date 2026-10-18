use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigRow {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub reference: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub enabled: bool,
    pub deleted: bool,
    pub delay_ms: Option<i64>,
    pub subscription_id: Option<i64>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub country: Option<String>,
    pub asn: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub fronting: Option<String>,
    pub updated_at: String,
    pub test: Option<ConfigTest>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigTest {
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
    pub failure_reason: Option<String>,
    pub tested_at: Option<String>,
}
