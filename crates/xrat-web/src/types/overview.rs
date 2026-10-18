use super::{ConfigRow, EventRow, SubscriptionRow};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Overview {
    #[serde(default)]
    pub dashboard: Option<DashboardData>,
    pub version: String,
    pub running: bool,
    pub traffic: Option<Traffic>,
    pub active_config: Option<i64>,
    pub configs: Vec<ConfigRow>,
    pub subscriptions: Vec<SubscriptionRow>,
    pub events: Vec<EventRow>,
    pub capabilities: Vec<Capability>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Traffic {
    pub uploaded_bytes: u64,
    pub downloaded_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrafficSample {
    pub sampled_at_ms: f64,
    pub traffic: Option<Traffic>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatencyMetric {
    Tcp,
    Icmp,
    RealDelay,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProbePoint {
    pub id: i64,
    pub tested_at: String,
    pub icmp_ms: Option<i64>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashboardData {
    pub sampled_at_ms: f64,
    pub session_id: Option<i64>,
    pub runtime_status: String,
    pub stats_enabled: bool,
    pub tun_enabled: bool,
    pub tun_active: bool,
    pub tun_available: bool,
    pub tun_reason: Option<String>,
    pub enabled_metrics: Vec<LatencyMetric>,
    pub last_subscription_refresh: Option<i64>,
    pub probes: Vec<ProbePoint>,
    #[serde(default)]
    pub traffic_history: Vec<TrafficSample>,
}
