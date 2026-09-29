use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, Response, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::json::JsonQuery;
use crate::server::auth::require_api_key;
use crate::server::{ServerResult, ServerState};

pub async fn b64(
    State(state): State<ServerState>,
    Query(query): Query<JsonQuery>,
) -> ServerResult<Response<Body>> {
    require_api_key(&state, query.key.as_deref())?;
    let raw_configs = state
        .services
        .configs
        .export_raw_configs(&query.to_request()?)
        .await?;

    let payload = raw_configs.join("\n");
    let encoded = STANDARD.encode(payload);
    let mut response = Response::new(Body::from(encoded));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    Ok(response)
}
