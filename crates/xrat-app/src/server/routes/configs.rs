use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::app::services::ConfigListRequest;
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigDetail, PaginatedResponse, detail_from_model};
use crate::server::{ServerError, ServerResult, ServerState};

#[derive(Debug, Deserialize)]
pub struct ConfigsQuery {
    pub key: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
}

pub async fn list_configs(
    State(state): State<ServerState>,
    Query(query): Query<ConfigsQuery>,
) -> ServerResult<Json<PaginatedResponse<ApiConfigDetail>>> {
    require_api_key(&state, query.key.as_deref())?;
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    if page == 0 {
        return Err(ServerError::InvalidQuery(
            "page must be greater than zero".to_string(),
        ));
    }
    if per_page == 0 || per_page > 200 {
        return Err(ServerError::InvalidQuery(
            "per_page must be between 1 and 200".to_string(),
        ));
    }

    let request = ConfigListRequest {
        only_enabled: query.enabled.unwrap_or(false),
        protocol: query.protocol.clone(),
        offset: Some(((page - 1) * per_page) as i64),
        limit: Some(per_page as i64),
        ..ConfigListRequest::default()
    };
    let result = state.services.configs.list(&request).await?;
    let items = result.items.iter().map(detail_from_model).collect();

    Ok(Json(PaginatedResponse {
        total: result.total as usize,
        page,
        per_page,
        items,
    }))
}

pub async fn get_config(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Query(query): Query<ConfigsQuery>,
) -> ServerResult<Json<ApiConfigDetail>> {
    require_api_key(&state, query.key.as_deref())?;
    let id = state
        .services
        .configs
        .resolve_id(&id)
        .await?
        .ok_or(ServerError::NotFound)?;
    let detail = state
        .services
        .configs
        .detail(id)
        .await?
        .ok_or(ServerError::NotFound)?;

    Ok(Json(detail_from_model(&detail)))
}
