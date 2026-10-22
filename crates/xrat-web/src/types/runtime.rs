use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeDetails {
    #[serde(default)]
    pub pac_available: bool,
    pub sampled_at_ms: f64,
    pub status: String,
    pub engine: String,
    pub process_id: Option<i64>,
    pub started_at: Option<String>,
    pub inbounds: Vec<RuntimeInbound>,
    pub engines: Vec<RuntimeEngine>,
    pub daemon_running: Option<bool>,
    pub rotation_enabled: bool,
    #[serde(default)]
    pub tun_enabled: bool,
    pub rotation_interval_secs: u64,
    pub running_jobs: usize,
    pub queued_jobs: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeInbound {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub reachable: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeEngine {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
}
