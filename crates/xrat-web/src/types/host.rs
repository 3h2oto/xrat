use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Engine {
    Xray,
    SingBox,
    V2ray,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub latest_version: Option<String>,
    pub update_available: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct HostUnlockRequest {
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HostUnlock {
    pub token: String,
    pub expires_at: i64,
}
