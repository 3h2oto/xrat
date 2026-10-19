use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventRow {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub created_at: String,
}
