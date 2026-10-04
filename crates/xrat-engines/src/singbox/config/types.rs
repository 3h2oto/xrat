use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxRoute {
    pub rules: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rule_set: Vec<serde_json::Value>,
    #[serde(rename = "final")]
    pub final_outbound: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_domain_resolver: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxLogConfig {
    pub level: String,
    /// sing-box omits timestamps by default. Enable them in generated configs so
    /// the TUI engine tab can parse a real time column for sing-box log lines.
    pub timestamp: bool,
}
