use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub username: String,
    pub kind: String,
    pub state: String,
    pub result: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub tests: Vec<ConfigTestProgress>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestMetric {
    Icmp,
    Tcp,
    RealDelay,
    Download,
    Upload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestMetricState {
    Queued,
    Running,
    Passed,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestMetricProgress {
    pub metric: TestMetric,
    pub state: TestMetricState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigTestProgress {
    pub config_id: i64,
    pub metrics: Vec<TestMetricProgress>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestJobProgress {
    pub message: String,
    pub tests: Vec<ConfigTestProgress>,
}
