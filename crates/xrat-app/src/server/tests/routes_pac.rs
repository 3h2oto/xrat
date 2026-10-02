use axum::body::to_bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, header};

use super::multi_config_state;
use crate::server::ServerError;
use crate::server::routes::pac;

#[tokio::test]
async fn pac_route_sets_content_type_and_is_unauthenticated() {
    // An API key is configured, but the PAC route must not require it.
    let state = multi_config_state(Some("secret"), 1).await;

    let response = pac::proxy_pac(State(state), host_headers("127.0.0.1:18203"))
        .await
        .expect("pac route should succeed without auth");

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .expect("content type should be set")
        .to_str()
        .expect("content type should be ascii");
    assert_eq!(content_type, "application/x-ns-proxy-autoconfig");

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    // No running runtime session, so the PAC routes everything DIRECT.
    assert!(text.contains("function FindProxyForURL"));
    assert!(text.contains("return \"DIRECT\";"));
}

#[tokio::test]
async fn pac_route_rejects_unallowed_host_header() {
    let state = multi_config_state(None, 1).await;

    let result = pac::proxy_pac(State(state), host_headers("evil.example")).await;

    assert!(matches!(result, Err(ServerError::PacHostNotAllowed)));
}

#[tokio::test]
async fn pac_route_returns_not_found_when_disabled() {
    let mut state = multi_config_state(None, 1).await;
    state.pac_enabled = false;

    let result = pac::proxy_pac(State(state), host_headers("127.0.0.1:18203")).await;

    assert!(matches!(result, Err(ServerError::NotFound)));
}

#[tokio::test]
async fn pac_route_accepts_configured_allowed_host() {
    let mut state = multi_config_state(None, 1).await;
    state.pac_allowed_hosts.push("pac.example.test".to_string());

    let response = pac::proxy_pac(State(state), host_headers("pac.example.test:18203"))
        .await
        .expect("configured host should be accepted");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

fn host_headers(host: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::HOST,
        HeaderValue::from_str(host).expect("host header should be valid"),
    );
    headers
}

#[tokio::test]
async fn pac_route_and_cli_use_case_render_the_same_active_session_and_rules() {
    let (context, _root) = crate::app::tests::TestAppBuilder::new("pac-active")
        .build_with_root()
        .await;
    let mut state = crate::server::ServerState::for_test(context.db.clone(), None);
    state.pac_rules.block_domains = vec!["blocked.example".into()];
    context
        .db
        .insert_runtime_session(&xrat_db::RuntimeSessionInsert {
            config_id: None,
            status: xrat_db::RuntimeSessionStatus::Running,
            http_host: Some("0.0.0.0".into()),
            http_port: Some(8080),
            socks_host: Some("127.0.0.1".into()),
            socks_port: Some(1080),
            shadowsocks_host: Some("127.0.0.1".into()),
            shadowsocks_port: Some(8388),
            process_id: None,
            failure_reason: None,
            started_at: None,
            stopped_at: None,
        })
        .await
        .unwrap();
    let active = crate::app::services::proxy_pac::active_endpoints(&context.db)
        .await
        .unwrap();
    assert_eq!(active.http, Some(("0.0.0.0".into(), 8080)));
    assert_eq!(active.shadowsocks, Some(("127.0.0.1".into(), 8388)));
    let expected = crate::app::services::proxy_pac::active_pac(&context.db, &state.pac_rules)
        .await
        .unwrap();
    let response = pac::proxy_pac(State(state), host_headers("localhost:18203"))
        .await
        .unwrap();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(body, expected);
    assert!(body.contains("SOCKS5 127.0.0.1:1080; PROXY 127.0.0.1:8080"));
    assert!(body.contains("blocked.example"));
    assert!(!body.contains("8388"));
}
