use serde::{Deserialize, Serialize};
use xrat_model::ConfigId;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeStages {
    pub skip_icmp: bool,
    pub skip_tcp: bool,
    pub skip_real_delay: bool,
    pub skip_download: bool,
    pub skip_upload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProbeRequest {
    Start {
        config_ids: Vec<ConfigId>,
        stages: ProbeStages,
    },
    Status {
        job_id: Option<String>,
    },
    Cancel {
        job_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeState {
    Running,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeSnapshot {
    pub job_id: String,
    pub state: ProbeState,
    pub run_id: Option<i64>,
    pub completed: usize,
    pub total: usize,
    pub last_config_id: Option<ConfigId>,
    pub error: Option<String>,
}
