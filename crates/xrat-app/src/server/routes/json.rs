use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::services::ConfigExportRequest;
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigSummary, summary_from_summary};
use crate::server::{ServerResult, ServerState};

#[derive(Debug, Deserialize)]
pub struct JsonQuery {
    pub key: Option<String>,
    pub top: Option<u32>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
}

impl JsonQuery {
    pub fn to_request(&self) -> ConfigExportRequest {
        ConfigExportRequest {
            enabled: self.enabled,
            protocol: self.protocol.clone(),
            top: self.top,
        }
    }
}

pub async fn json(
    State(state): State<ServerState>,
    Query(query): Query<JsonQuery>,
) -> ServerResult<Json<Vec<ApiConfigSummary>>> {
    require_api_key(&state, query.key.as_deref())?;
    let result = state
        .services
        .configs
        .export_summaries(&query.to_request())
        .await?;
    Ok(Json(result.iter().map(summary_from_summary).collect()))
}
