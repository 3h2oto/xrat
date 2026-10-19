use super::security::{Principal, require_operator};
use crate::server::{ServerError, ServerResult, ServerState};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
};
use xrat_web::types::{SubscriptionSource, SubscriptionSourcePreview};

async fn source_url(state: &ServerState, principal: &Principal, id: i64) -> ServerResult<String> {
    require_operator(principal)?;
    let subscription = state
        .db
        .get_subscription_by_id(id.into())
        .await?
        .ok_or(ServerError::NotFound)?;
    if subscription.source_kind != "url" {
        return Err(ServerError::InvalidQuery(
            "This subscription has no source URL.".into(),
        ));
    }
    subscription.source_url.ok_or(ServerError::NotFound)
}

pub async fn source(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<i64>,
) -> ServerResult<impl IntoResponse> {
    Ok((
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::REFERRER_POLICY, "no-referrer"),
        ],
        Json(SubscriptionSource {
            url: source_url(&state, &principal, id).await?,
        }),
    ))
}

pub async fn preview(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<i64>,
) -> ServerResult<Json<SubscriptionSourcePreview>> {
    let url = source_url(&state, &principal, id).await?;
    Ok(Json(SubscriptionSourcePreview {
        qr_supported: qrcode::QrCode::with_error_correction_level(url, qrcode::EcLevel::H).is_ok(),
    }))
}
