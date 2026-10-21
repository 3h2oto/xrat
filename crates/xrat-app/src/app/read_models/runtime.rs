use crate::app::daemon::ipc::TunStatePayload;
use crate::app::runtime_service::RuntimeStatusSnapshot;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RuntimeOverview {
    pub runtime: RuntimeStatusSnapshot,
    pub tun: TunStatePayload,
    pub daemon_ready: bool,
    pub rotation_enabled: bool,
    pub rotation_interval_secs: u64,
    pub test_stages: Vec<String>,
    pub config_path: String,
}
