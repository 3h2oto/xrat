use axum::body::to_bytes;
use axum::extract::{Query, State};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::{populated_state, test_node};
use crate::server::routes::{b64, json::JsonQuery};

#[tokio::test]
async fn b64_route_returns_subscription_text_payload() {
    let state = populated_state(None).await;

    let response = b64::b64(
        State(state),
        Query(JsonQuery {
            key: None,
            top: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("b64 route should succeed");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let decoded = STANDARD.decode(body).expect("body should be valid base64");

    assert_eq!(
        String::from_utf8(decoded).expect("utf8"),
        test_node().raw_config
    );
}

#[tokio::test]
async fn subscription_and_json_top_exports_select_the_same_ordered_configs() {
    let state = super::multi_config_state(None, 4).await;
    let query = || JsonQuery {
        key: None,
        top: Some(2),
        enabled: None,
        protocol: Some("vless".into()),
    };
    let axum::Json(summaries) =
        crate::server::routes::json::json(State(state.clone()), Query(query()))
            .await
            .unwrap();
    let response = b64::b64(State(state.clone()), Query(query()))
        .await
        .unwrap();
    assert_eq!(
        response.headers()[axum::http::header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let decoded = String::from_utf8(STANDARD.decode(body).unwrap()).unwrap();
    let mut expected = Vec::new();
    for summary in summaries {
        expected.push(
            state
                .db
                .get_config_by_id(summary.id)
                .await
                .unwrap()
                .unwrap()
                .raw_config,
        );
    }
    assert_eq!(decoded, expected.join("\n"));
}

#[tokio::test]
async fn subscription_export_keeps_auth_before_top_validation() {
    let state = super::multi_config_state(Some("secret"), 0).await;
    let query = |key| JsonQuery {
        key,
        top: Some(201),
        enabled: None,
        protocol: None,
    };
    assert!(matches!(
        b64::b64(State(state.clone()), Query(query(Some("wrong".into())))).await,
        Err(crate::server::ServerError::InvalidApiKey)
    ));
    assert!(matches!(
        b64::b64(State(state), Query(query(Some("secret".into())))).await,
        Err(crate::server::ServerError::Application(
            crate::app::AppError::InvalidArgument(_)
        ))
    ));
}
