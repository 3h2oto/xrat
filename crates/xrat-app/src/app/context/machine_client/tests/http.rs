use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn request(
    router: &axum::Router,
    path: &str,
    method: &str,
    body: serde_json::Value,
    cookie: &str,
    csrf: &str,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .method(method)
                .header("content-type", "application/json")
                .header("host", "127.0.0.1:18203")
                .header("origin", "http://127.0.0.1:18203")
                .header("cookie", cookie)
                .header("x-csrf-token", csrf)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn machine_frontend_http_preserves_auth_csrf_roles_and_service_catalog() {
    use crate::app::{
        config::SecretString,
        tests::{
            TestAppBuilder,
            fixtures::{test_node, test_source},
        },
    };
    let (mut owner, _owner_root) = TestAppBuilder::new("machine-http").build_with_root().await;
    owner.app_config.routing.block.domain = vec!["full:blocked.machine.example".into()];
    owner
        .db
        .import_nodes(&test_source(), &[test_node("owner.example")])
        .await
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let mut context = compose(loopback(&owner, true).await, &root.path().join("ui"))
        .await
        .unwrap();
    context.app_config.server.web.enabled = true;
    context.app_config.server.web.admin_username = "admin".into();
    context.app_config.server.web.admin_password_hash = Some(SecretString::Literal(
        crate::server::hash_password("test administrator password").unwrap(),
    ));
    let router = crate::server::build_router_from_context(
        &context,
        &context.app_config.server,
        &context.app_config.routing,
    )
    .unwrap();
    for path in [
        "/json",
        "/b64",
        "/configs",
        "/configs/1",
        "/b64?key=unconfigured",
    ] {
        assert_eq!(
            request(&router, path, "GET", serde_json::Value::Null, "", "")
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    context.app_config.server.key = Some(SecretString::Literal("test-key".into()));
    let authenticated_exports = crate::server::build_router_from_context(
        &context,
        &context.app_config.server,
        &context.app_config.routing,
    )
    .unwrap();
    assert_eq!(
        request(
            &authenticated_exports,
            "/configs?key=test-key",
            "GET",
            serde_json::Value::Null,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_ne!(
        request(
            &authenticated_exports,
            "/b64?key=test-key&token=invalid",
            "GET",
            serde_json::Value::Null,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/overview",
            "GET",
            serde_json::Value::Null,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let response = request(
        &router,
        "/api/v1/login",
        "POST",
        serde_json::json!({"username":"admin","password":"test administrator password"}),
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let me: xrat_web::types::LoginResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    let response = request(
        &router,
        "/api/v1/overview",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let overview: xrat_web::types::Overview =
        serde_json::from_slice(&to_bytes(response.into_body(), 1048576).await.unwrap()).unwrap();
    assert_eq!(overview.configs.len(), 1);
    assert!(
        overview
            .capabilities
            .iter()
            .any(|capability| capability.name == "host_maintenance" && !capability.available)
    );
    let id = owner.db.list_configs(&Default::default()).await.unwrap()[0].id;
    let mutation = serde_json::json!({"action":"disable","id":id.0});
    assert_eq!(
        request(
            &router,
            "/api/v1/actions",
            "POST",
            mutation.clone(),
            &cookie,
            ""
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let response = request(
        &router,
        "/api/v1/actions",
        "POST",
        mutation,
        &cookie,
        &me.csrf_token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if !owner
                .db
                .get_config_by_id(id)
                .await
                .unwrap()
                .unwrap()
                .is_enabled
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(context.db.get_config_count().await.unwrap(), 0);
    let response = request(
        &router,
        "/proxy.pac",
        "GET",
        serde_json::Value::Null,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let pac = to_bytes(response.into_body(), 1048576).await.unwrap();
    assert!(String::from_utf8_lossy(&pac).contains("blocked.machine.example"));
    assert!(
        !std::fs::read_to_string(&context.runtime_paths.config_path)
            .unwrap()
            .contains("blocked.machine.example")
    );

    context
        .db
        .web_save_user(&xrat_db::WebUser {
            username: "viewer".into(),
            role: "viewer".into(),
            password_hash: crate::server::hash_password("test viewer password").unwrap(),
            credential_version: "v1".into(),
            enabled: true,
            must_change_password: false,
        })
        .await
        .unwrap();
    let response = request(
        &router,
        "/api/v1/login",
        "POST",
        serde_json::json!({"username":"viewer","password":"test viewer password"}),
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let me: xrat_web::types::LoginResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert_eq!(
        request(
            &router,
            "/api/v1/actions",
            "POST",
            serde_json::json!({"action":"enable","id":id.0}),
            &cookie,
            &me.csrf_token
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert!(
        !owner
            .db
            .get_config_by_id(id)
            .await
            .unwrap()
            .unwrap()
            .is_enabled
    );
    assert!(owner.db.web_users().await.unwrap().is_empty());
}
