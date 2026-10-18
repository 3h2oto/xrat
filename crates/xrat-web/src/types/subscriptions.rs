use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRow {
    pub id: i64,
    pub name: String,
    pub source: String,
    pub config_count: i64,
    pub created_at: String,
    pub updated_at: String,
}
