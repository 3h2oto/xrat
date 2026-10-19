use crate::{
    app::services::ConfigListRequest,
    server::{
        ServerError, ServerResult, ServerState,
        web::{
            rows,
            security::{Principal, require_operator},
        },
    },
};
use axum::{Extension, Json, extract::State};
use std::collections::{HashMap, HashSet};
use xrat_web::configs::sharing::{
    ConfigFilter, ConfigShareRequest, ConfigShareResponse, filter_configs,
};

pub async fn share(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<ConfigShareRequest>,
) -> ServerResult<Json<ConfigShareResponse>> {
    require_operator(&principal)?;
    Ok(Json(export(&state, &request).await?))
}

pub async fn preview(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<ConfigShareRequest>,
) -> ServerResult<Json<xrat_web::configs::sharing::ConfigSharePreview>> {
    require_operator(&principal)?;
    let exported = export(&state, &request).await?;
    Ok(Json(xrat_web::configs::sharing::ConfigSharePreview {
        count: exported.count,
        qr_supported: exported.count > 0
            && qrcode::QrCode::with_error_correction_level(&exported.raw, qrcode::EcLevel::H)
                .is_ok(),
    }))
}

pub(crate) async fn export(
    state: &ServerState,
    request: &ConfigShareRequest,
) -> ServerResult<ConfigShareResponse> {
    let ids = match request {
        ConfigShareRequest::Selected { ids } => {
            if ids.is_empty()
                || ids.len() > 500
                || ids.iter().any(|id| *id <= 0)
                || ids.iter().collect::<HashSet<_>>().len() != ids.len()
            {
                return Err(ServerError::InvalidQuery(
                    "Select 1–500 unique configs".into(),
                ));
            }
            ids.clone()
        }
        ConfigShareRequest::Filtered { filter } => {
            filter
                .validate()
                .map_err(|message| ServerError::InvalidQuery(message.into()))?;
            let configs = state
                .services
                .configs
                .list(&ConfigListRequest {
                    include_deleted: true,
                    ..Default::default()
                })
                .await?
                .items
                .into_iter()
                .map(rows::config)
                .collect::<Vec<_>>();
            let subs = state
                .db
                .list_subscriptions()
                .await?
                .into_iter()
                .map(rows::subscription)
                .collect::<Vec<_>>();
            filter_configs(&configs, &subs, filter, None)
                .into_iter()
                .map(|row| row.id)
                .collect()
        }
    };
    if ids.is_empty() {
        return Ok(ConfigShareResponse {
            count: 0,
            raw: String::new(),
        });
    }
    if ids.len() == 1 {
        let config = state
            .db
            .get_config_by_id(ids[0].into())
            .await?
            .ok_or(ServerError::NotFound)?;
        return Ok(ConfigShareResponse {
            count: 1,
            raw: config.raw_config,
        });
    }
    let mut raw = state
        .db
        .list_configs(&xrat_db::ConfigListFilter {
            include_deleted: true,
            ..Default::default()
        })
        .await?
        .into_iter()
        .map(|row| (row.id.0, row.raw_config))
        .collect::<HashMap<_, _>>();
    let lines = ids
        .iter()
        .map(|id| raw.remove(id).ok_or(ServerError::NotFound))
        .collect::<ServerResult<Vec<_>>>()?;
    Ok(ConfigShareResponse {
        count: lines.len(),
        raw: lines.join("\n"),
    })
}

pub(crate) async fn filtered_export(
    state: &ServerState,
    filter: ConfigFilter,
) -> ServerResult<String> {
    Ok(export(state, &ConfigShareRequest::Filtered { filter })
        .await?
        .raw)
}
