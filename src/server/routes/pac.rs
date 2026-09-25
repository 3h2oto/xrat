use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Response, header};

use crate::app::services::proxy_pac::{PacEndpoints, render_pac};
use crate::server::{ServerError, ServerResult, ServerState};
/// `GET /proxy.pac` — unauthenticated local helper. PAC consumers (browsers,
/// desktop proxy settings) usually cannot send auth headers, and the file only
/// exposes non-secret local endpoint data.
pub async fn proxy_pac(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> ServerResult<Response<Body>> {
    if !state.pac_enabled {
        return Err(ServerError::NotFound);
    }
    require_allowed_pac_host(&state, &headers)?;

    let endpoints = active_endpoints(&state).await?;
    let body = render_pac(&endpoints, &state.pac_rules);

    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ns-proxy-autoconfig"),
    );
    Ok(response)
}

fn require_allowed_pac_host(state: &ServerState, headers: &HeaderMap) -> ServerResult<()> {
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(host_without_port)
    else {
        return Err(ServerError::PacHostNotAllowed);
    };

    let allowed = state
        .pac_allowed_hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(host));
    if allowed {
        Ok(())
    } else {
        Err(ServerError::PacHostNotAllowed)
    }
}

fn host_without_port(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(rest) = value.strip_prefix('[') {
        return rest.split_once(']').map(|(host, _)| host);
    }
    Some(value.split_once(':').map_or(value, |(host, _)| host))
}

async fn active_endpoints(state: &ServerState) -> ServerResult<PacEndpoints> {
    let Some(session) = state.db.get_running_runtime_session().await? else {
        return Ok(PacEndpoints::default());
    };

    let http = inbound(session.http_host.as_deref(), session.http_port);
    let socks = inbound(session.socks_host.as_deref(), session.socks_port);

    Ok(PacEndpoints { http, socks })
}

fn inbound(host: Option<&str>, port: Option<i64>) -> Option<(String, u16)> {
    match (host, port) {
        (Some(host), Some(port)) if (1..=i64::from(u16::MAX)).contains(&port) => {
            Some((host.to_string(), port as u16))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_host_without_port() {
        assert_eq!(host_without_port("localhost:18203"), Some("localhost"));
        assert_eq!(host_without_port("127.0.0.1:18203"), Some("127.0.0.1"));
        assert_eq!(host_without_port("[::1]:18203"), Some("::1"));
    }
}
