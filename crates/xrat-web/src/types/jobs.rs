use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub username: String,
    pub kind: String,
    pub state: String,
    pub result: Option<String>,
    pub created_at: i64,
}
