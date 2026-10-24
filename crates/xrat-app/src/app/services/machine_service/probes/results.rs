use crate::app::services::testing::{TestOutputRow, TestStatus};
use serde::{Deserialize, Serialize};
use xrat_model::ConfigId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    #[serde(with = "WireOutput")]
    pub(crate) output: TestOutputRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResultPage {
    pub total: u32,
    pub rows: Vec<ProbeResult>,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "TestOutputRow")]
struct WireOutput {
    id: ConfigId,
    r#ref: String,
    name: Option<String>,
    protocol: String,
    address: String,
    port: i64,
    icmp_ms: Option<u32>,
    real_delay_ms: Option<u32>,
    download_mbps: Option<f64>,
    upload_mbps: Option<f64>,
    status: TestStatus,
    error: Option<String>,
    tcp_ms: Option<u32>,
    ttfb_ms: Option<u32>,
    http_status: Option<u16>,
    dial_endpoint_ip: Option<String>,
    dial_endpoint_location: Option<String>,
    dial_endpoint_country: Option<String>,
    dial_endpoint_asn: Option<String>,
    dial_endpoint_geoip_source: Option<String>,
    dial_endpoint_fronting: Option<String>,
    ran_icmp: bool,
    ran_tcp: bool,
    ran_real_delay: bool,
    icmp_ok: bool,
    tcp_ok: bool,
    real_delay_ok: bool,
    failure_kind: Option<String>,
    elapsed_secs: f64,
}
