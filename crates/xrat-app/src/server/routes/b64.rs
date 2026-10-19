use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, Response, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::json::JsonQuery;
use crate::server::auth::require_api_key;
use crate::server::web::security::subscription_principal;
use crate::server::{ServerResult, ServerState};
use xrat_web::configs::sharing::ConfigFilter;

#[derive(Default, serde::Deserialize)]
#[serde(default)]
pub struct B64Query {
    pub key: Option<String>,
    pub token: Option<String>,
    pub top: Option<u32>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
    pub q: Option<String>,
    pub state: Option<String>,
    pub subscription: Option<String>,
    pub test: Option<String>,
    pub country: Option<String>,
    pub max_delay: Option<u64>,
    pub show_deleted: Option<bool>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl From<JsonQuery> for B64Query {
    fn from(query: JsonQuery) -> Self {
        Self {
            key: query.key,
            top: query.top,
            enabled: query.enabled,
            protocol: query.protocol,
            ..Default::default()
        }
    }
}

pub async fn b64(
    State(state): State<ServerState>,
    Query(query): Query<B64Query>,
) -> ServerResult<Response<Body>> {
    if let Some(token) = &query.token {
        let principal = subscription_principal(&state, token).await?;
        if !principal.user.role.can_operate() || principal.user.must_change_password {
            return Err(crate::server::ServerError::Forbidden);
        }
    } else {
        require_api_key(&state, query.key.as_deref())?;
    }
    let table_filters = query.q.is_some()
        || query.state.is_some()
        || query.subscription.is_some()
        || query.test.is_some()
        || query.country.is_some()
        || query.max_delay.is_some()
        || query.show_deleted.is_some()
        || query.sort.is_some()
        || query.order.is_some();
    let payload = if table_filters {
        let top = query
            .top
            .map(crate::app::services::validate_top)
            .transpose()?;
        let mut filter = ConfigFilter {
            q: query.q.unwrap_or_default(),
            protocol: query.protocol.unwrap_or_default(),
            state: query.state.unwrap_or_default(),
            subscription: query.subscription.unwrap_or_default(),
            test: query.test.unwrap_or_default(),
            country: query.country.unwrap_or_default(),
            max_delay: query.max_delay,
            show_deleted: query.show_deleted.unwrap_or(false),
            sort: query.sort.unwrap_or_default(),
            order: query.order.unwrap_or_default(),
        };
        if query.enabled.unwrap_or(true) && (filter.state.is_empty() || filter.state == "all") {
            filter.state = "enabled".into();
        }
        filter
            .validate()
            .map_err(|message| crate::server::ServerError::InvalidQuery(message.into()))?;
        if top.is_some() {
            filter.sort = "delay".into();
            filter.order = "asc".into();
            filter.max_delay = Some(filter.max_delay.unwrap_or(u64::MAX));
        }
        let raw = super::sharing::filtered_export(&state, filter).await?;
        if let Some(top) = top {
            raw.lines()
                .take(top as usize)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            raw
        }
    } else {
        state
            .services
            .configs
            .export_subscription(
                &JsonQuery {
                    key: None,
                    top: query.top,
                    enabled: query.enabled,
                    protocol: query.protocol,
                }
                .to_request(),
            )
            .await?
    };

    let encoded = STANDARD.encode(payload);
    let mut response = Response::new(Body::from(encoded));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok(response)
}
