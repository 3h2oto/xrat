use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Url,
    File,
    RawText,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::File => "file",
            Self::RawText => "raw_text",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportSource {
    pub kind: SourceKind,
    pub value: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImportSummary {
    pub subscription_id: xrat_model::SubscriptionId,
    pub imported_configs: usize,
    pub removed_configs: u64,
    pub total_configs: i64,
}

/// A URL-backed subscription eligible for automatic refresh.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshableSubscription {
    pub id: xrat_model::SubscriptionId,
    pub source_url: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionRecord {
    pub id: xrat_model::SubscriptionId,
    pub r#ref: String,
    pub source_kind: String,
    pub source_url: Option<String>,
    pub name: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
    pub config_count: i64,
}
