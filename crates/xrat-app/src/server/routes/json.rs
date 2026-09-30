use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::services::{ConfigListRequest, validate_top};
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigSummary, summary_from_summary};
use crate::server::{ServerError, ServerResult, ServerState};

const DEFAULT_ENABLED_ONLY: bool = true;

#[derive(Debug, Deserialize)]
pub struct JsonQuery {
    pub key: Option<String>,
    pub top: Option<u32>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
}

impl JsonQuery {
    /// Translate query parameters into a shared config list request.
    pub fn to_request(&self) -> ServerResult<ConfigListRequest> {
        let top_by_real_delay = match self.top {
            Some(top) => Some(validate_top(top).map_err(ServerError::from)?),
            None => None,
        };
        Ok(ConfigListRequest {
            only_enabled: self.enabled.unwrap_or(DEFAULT_ENABLED_ONLY),
            protocol: self.protocol.clone(),
            top_by_real_delay,
            ..ConfigListRequest::default()
        })
    }
}

pub async fn json(
    State(state): State<ServerState>,
    Query(query): Query<JsonQuery>,
) -> ServerResult<Json<Vec<ApiConfigSummary>>> {
    require_api_key(&state, query.key.as_deref())?;
    let result = state.services.configs.list(&query.to_request()?).await?;
    Ok(Json(
        result
            .items
            .iter()
            .map(|detail| summary_from_summary(&detail.summary))
            .collect(),
    ))
}
