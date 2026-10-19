use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use xrat_web::configs::sharing::{ConfigFilter, ConfigShareRequest, ConfigShareResponse};

async fn token(router: &axum::Router, cookie: &str, csrf: &str) -> xrat_web::types::CreatedToken {
    let response = request(
        router,
        "/api/v1/tokens",
        "POST",
        serde_json::json!({"name":"subscription", "scope":"read"}),
        cookie,
        csrf,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap()
}
async fn decoded(router: &axum::Router, path: &str) -> String {
    let response = request(router, path, "GET", serde_json::Value::Null, "", "", "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    String::from_utf8(
        STANDARD
            .decode(to_bytes(response.into_body(), 1_048_576).await.unwrap())
            .unwrap(),
    )
    .unwrap()
}

#[tokio::test]
async fn subscription_query_tokens_obey_revocation_roles_and_credential_versions_without_legacy_fallback()
 {
    let (router, state) = fixture().await;
    tables::seed(&state).await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let path = format!("/b64?token={}&enabled=false&state=all", token.token);
    assert_eq!(decoded(&router, &path).await.lines().count(), 2);
    // A missing token may use legacy public export; an invalid supplied token may never do so.
    state
        .db
        .web_delete_token(&super::super::security::digest(&token.token), "admin")
        .await
        .unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    state
        .db
        .web_save_user(&xrat_db::WebUser {
            username: "share-user".into(),
            role: "operator".into(),
            password_hash: hash_password("test password long").unwrap(),
            credential_version: "v1".into(),
            enabled: true,
            must_change_password: false,
        })
        .await
        .unwrap();
    let (cookie, me) = login(&router, "share-user", "test password long").await;
    let token = self::token(&router, &cookie, &me.csrf_token).await;
    let path = format!("/b64?token={}", token.token);
    assert_eq!(decoded(&router, &path).await.lines().count(), 2);
    let mut record = super::super::security::user(&state, "share-user")
        .await
        .unwrap()
        .unwrap();
    record.role = "viewer".into();
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    record.role = "operator".into();
    record.must_change_password = true;
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    record.must_change_password = false;
    record.enabled = false;
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    record.enabled = true;
    record.credential_version = "v2".into();
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn share_exports_match_subscription_filters_and_selected_ids_including_deleted() {
    let (router, state) = fixture().await;
    let ids = tables::seed(&state).await;
    state.db.delete_config(ids[0].into()).await.unwrap();
    state
        .db
        .set_config_enabled(ids[1].into(), false)
        .await
        .unwrap();
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let filter = ConfigFilter {
        show_deleted: true,
        sort: "name".into(),
        order: "desc".into(),
        ..Default::default()
    };
    let response = request(
        &router,
        "/api/v1/configs/share",
        "POST",
        serde_json::to_value(ConfigShareRequest::Filtered {
            filter: filter.clone(),
        })
        .unwrap(),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let exported: ConfigShareResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    assert_eq!(exported.count, 2);
    assert!(exported.raw.lines().next().unwrap().contains("two.example"));
    let path = format!(
        "/b64?token={}&enabled=false&show_deleted=true&sort=name&order=desc",
        token.token
    );
    assert_eq!(decoded(&router, &path).await, exported.raw);
    let path = format!("/b64?token={}&enabled=false&state=disabled", token.token);
    let disabled = decoded(&router, &path).await;
    assert_eq!(disabled.lines().count(), 1);
    assert!(disabled.contains("two.example"));
    let path = format!(
        "/b64?token={}&enabled=false&state=deleted&show_deleted=true",
        token.token
    );
    assert!(decoded(&router, &path).await.contains("one.example"));
    let response = request(
        &router,
        "/api/v1/configs/share",
        "POST",
        serde_json::json!({"scope":"selected", "ids": [ids[1], ids[0]]}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let selected: ConfigShareResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    assert_eq!(selected.raw, exported.raw);
    for bad in [
        serde_json::json!({"scope":"selected","ids":[]}),
        serde_json::json!({"scope":"selected","ids":[ids[0],ids[0]]}),
        serde_json::json!({"scope":"filtered","filter":{"test":"typo"}}),
    ] {
        assert_eq!(
            request(
                &router,
                "/api/v1/configs/share",
                "POST",
                bad,
                &cookie,
                &me.csrf_token,
                "http://127.0.0.1:18203"
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &router,
            "/api/v1/configs/share",
            "POST",
            serde_json::json!({"scope":"selected","ids":[ids[0]]}),
            &cookie,
            "",
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    state.db.restore_config(ids[0].into()).await.unwrap();
    let path = format!(
        "/b64?token={}&enabled=false&state=deleted&show_deleted=true",
        token.token
    );
    assert!(decoded(&router, &path).await.is_empty());
}

#[tokio::test]
async fn subscription_accepts_legacy_keys_and_account_tokens_with_token_precedence() {
    let (_, mut state) = fixture().await;
    state.api_key = Some("legacy-secret".into());
    let router = crate::server::build_router(state.clone());
    tables::seed(&state).await;
    assert_eq!(
        request(&router, "/b64", "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        decoded(&router, "/b64?key=legacy-secret&enabled=false&state=all")
            .await
            .lines()
            .count(),
        2
    );
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let url = format!(
        "/b64?token={}&key=wrong&enabled=false&state=all",
        token.token
    );
    assert_eq!(decoded(&router, &url).await.lines().count(), 2);
    for url in [
        "/b64?token=wrong&key=legacy-secret",
        "/b64?token=&key=legacy-secret",
    ] {
        assert_eq!(
            request(&router, url, "GET", serde_json::Value::Null, "", "", "")
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn selected_read_token_mints_subscription_only_credentials_and_revocation_cascades() {
    let (router, state) = fixture().await;
    tables::seed(&state).await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    token(&router, &cookie, &me.csrf_token).await;
    let parent = state.db.web_tokens("admin").await.unwrap().remove(0);
    let endpoint = format!("/api/v1/tokens/{}/subscription", parent.token_hash);
    assert_eq!(
        request(
            &router,
            &endpoint,
            "POST",
            serde_json::Value::Null,
            &cookie,
            "",
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut secrets = Vec::new();
    for _ in 0..2 {
        let response = request(
            &router,
            &endpoint,
            "POST",
            serde_json::Value::Null,
            &cookie,
            &me.csrf_token,
            "http://127.0.0.1:18203",
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let created: xrat_web::types::CreatedToken =
            serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
        assert!(created.token.starts_with("sub_"));
        assert_eq!(
            decoded(&router, &format!("/b64?token={}", created.token))
                .await
                .lines()
                .count(),
            2
        );
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/overview")
                    .header("authorization", format!("Bearer {}", created.token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        secrets.push(created.token);
    }
    assert_ne!(secrets[0], secrets[1]);
    let mut other = parent.clone();
    other.token_hash = "other-owner-token".into();
    other.username = "someone-else".into();
    state.db.web_save_token(&other).await.unwrap();
    other.token_hash = "manage-token".into();
    other.username = "admin".into();
    other.scope = "manage".into();
    state.db.web_save_token(&other).await.unwrap();
    for id in ["other-owner-token", "manage-token", "not-a-token"] {
        assert_eq!(
            request(
                &router,
                &format!("/api/v1/tokens/{id}/subscription"),
                "POST",
                serde_json::Value::Null,
                &cookie,
                &me.csrf_token,
                "http://127.0.0.1:18203"
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    state
        .db
        .web_delete_token(&parent.token_hash, "admin")
        .await
        .unwrap();
    for secret in secrets {
        assert_eq!(
            request(
                &router,
                &format!("/b64?token={secret}"),
                "GET",
                serde_json::Value::Null,
                "",
                "",
                ""
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn qr_preview_reports_capacity_without_exposing_raw_configs() {
    let (router, state) = fixture().await;
    let ids = tables::seed(&state).await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/configs/share/preview",
        "POST",
        serde_json::json!({"scope":"selected", "ids":[ids[0]]}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
    let preview: xrat_web::configs::sharing::ConfigSharePreview =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(preview.count, 1);
    assert!(preview.qr_supported);
    assert!(!String::from_utf8_lossy(&bytes).contains("vless://"));
    let raw = (0..30).map(|i| format!("vless://00000000-0000-0000-0000-{i:012}@node-{i}.example:443?security=tls#LongConfig{}", "x".repeat(100))).collect::<Vec<_>>().join("\n");
    state
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::RawText,
                value: "qr-capacity-fixture".into(),
                name: None,
            },
            &xrat_config::parse_text(&raw),
        )
        .await
        .unwrap();
    let response = request(
        &router,
        "/api/v1/configs/share/preview",
        "POST",
        serde_json::json!({"scope":"filtered", "filter":{}}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let preview: xrat_web::configs::sharing::ConfigSharePreview =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert!(!preview.qr_supported);
    assert!(preview.count > 30);
}
