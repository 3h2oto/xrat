use super::*;
use xrat_web::types::{Action, BulkOperation, Overview};

pub(super) async fn seed(state: &ServerState) -> Vec<i64> {
    let context = state.web.as_ref().unwrap().context.as_ref().unwrap();
    std::fs::create_dir_all(context.runtime_paths.config_path.parent().unwrap()).unwrap();
    std::fs::write(&context.runtime_paths.config_path, "").unwrap();
    let nodes = xrat_config::parse_text(
        "vless://11111111-1111-1111-1111-111111111111@one.example:443?security=tls#One\nvless://22222222-2222-2222-2222-222222222222@two.example:443?security=tls#Two",
    );
    state
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::RawText,
                value: "table-fixture".into(),
                name: Some("Fixture".into()),
            },
            &nodes,
        )
        .await
        .unwrap();
    state
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.id.0)
        .collect()
}
#[tokio::test]
async fn bulk_actions_require_operator_csrf_and_bounded_unique_ids() {
    let (router, state) = fixture().await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let origin = "http://127.0.0.1:18203";
    for ids in [vec![], vec![1, 1], vec![0], (1..=501).collect()] {
        let body = serde_json::to_value(Action::Bulk {
            operation: BulkOperation::Disable,
            ids,
        })
        .unwrap();
        assert_eq!(
            request(
                &router,
                "/api/v1/actions",
                "POST",
                body,
                &cookie,
                &me.csrf_token,
                origin
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let body = serde_json::to_value(Action::Bulk {
        operation: BulkOperation::Disable,
        ids: vec![1],
    })
    .unwrap();
    assert_eq!(
        request(
            &router,
            "/api/v1/actions",
            "POST",
            body.clone(),
            &cookie,
            "",
            origin
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    state
        .db
        .web_save_user(&xrat_db::WebUser {
            username: "viewer".into(),
            role: "viewer".into(),
            password_hash: hash_password("viewer password").unwrap(),
            credential_version: "v1".into(),
            enabled: true,
            must_change_password: false,
        })
        .await
        .unwrap();
    let (cookie, me) = login(&router, "viewer", "viewer password").await;
    assert_eq!(
        request(
            &router,
            "/api/v1/actions",
            "POST",
            body,
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert!(
        serde_json::from_value::<Action>(
            serde_json::json!({"action":"bulk", "operation":"purge", "ids":[1]})
        )
        .is_err()
    );
}
#[tokio::test]
async fn bulk_job_mutates_all_selected_configs_and_preserves_soft_delete_restore() {
    let (router, state) = fixture().await;
    let ids = seed(&state).await;
    assert_eq!(ids.len(), 2);
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    for operation in [
        BulkOperation::Disable,
        BulkOperation::Enable,
        BulkOperation::Delete,
        BulkOperation::Restore,
    ] {
        let body = serde_json::to_value(Action::Bulk {
            operation,
            ids: ids.clone(),
        })
        .unwrap();
        let response = request(
            &router,
            "/api/v1/actions",
            "POST",
            body,
            &cookie,
            &me.csrf_token,
            "http://127.0.0.1:18203",
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let job: xrat_web::types::Job =
            serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
        let mut completed = false;
        for _ in 0..200 {
            let current = state
                .db
                .web_jobs()
                .await
                .unwrap()
                .into_iter()
                .find(|row| row.id == job.id)
                .unwrap();
            if current.state != "running" {
                assert_eq!(current.state, "succeeded", "{:?}", current.result);
                completed = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(completed);
        for id in &ids {
            let row = state
                .db
                .get_config_by_id((*id).into())
                .await
                .unwrap()
                .unwrap();
            match operation {
                BulkOperation::Disable => assert!(!row.is_enabled),
                BulkOperation::Enable => assert!(row.is_enabled),
                BulkOperation::Delete => assert!(row.is_deleted),
                BulkOperation::Restore => assert!(!row.is_deleted),
                _ => unreachable!(),
            }
        }
    }
    let context = state.web.as_ref().unwrap().context.as_ref().unwrap();
    let flag = xrat_support::cancel::CancellationFlag::new();
    flag.cancel();
    crate::app::services::web_management::execute(
        context,
        Action::Bulk {
            operation: BulkOperation::Delete,
            ids: ids.clone(),
        },
        &flag,
        "cancelled-test",
    )
    .await
    .unwrap();
    for id in ids {
        assert!(
            !state
                .db
                .get_config_by_id(id.into())
                .await
                .unwrap()
                .unwrap()
                .is_deleted
        );
    }
}
#[tokio::test]
async fn overview_exposes_table_metadata_without_raw_credentials() {
    let (router, state) = fixture().await;
    let ids = seed(&state).await;
    state
        .db
        .insert_connection_test(&xrat_db::ConnectionTestInsert {
            run_id: None,
            config_id: ids[0].into(),
            icmp_ok: Some(false),
            icmp_ms: None,
            tcp_ok: Some(true),
            tcp_ms: Some(12),
            real_delay_ok: Some(false),
            real_delay_ms: None,
            download_mbps: Some(25.5),
            upload_mbps: Some(4.2),
            connect_ms: Some(9),
            ttfb_ms: Some(20),
            http_status: Some(503),
            dial_endpoint_ip: None,
            dial_endpoint_location: Some("DE/Hesse/Eschborn".into()),
            dial_endpoint_country: Some("DE".into()),
            dial_endpoint_asn: Some("AS64500".into()),
            dial_endpoint_geoip_source: None,
            dial_endpoint_fronting: Some("gcore".into()),
            failure_kind: Some("proxy".into()),
            failure_reason: Some("HTTP status 503".into()),
        })
        .await
        .unwrap();
    let (cookie, _) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/overview",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let data: Overview = serde_json::from_slice(&bytes).unwrap();
    let row = data.configs.iter().find(|row| row.id == ids[0]).unwrap();
    let test = row.test.as_ref().unwrap();
    assert_eq!(test.tcp_ms, Some(12));
    assert_eq!(test.real_delay_ok, Some(false));
    assert_eq!(test.download_mbps, Some(25.5));
    assert_eq!(row.country.as_deref(), Some("DE"));
    assert_eq!(row.location.as_deref(), Some("DE/Hesse/Eschborn"));
    assert_eq!(row.fronting.as_deref(), Some("gcore"));
    assert!(!row.reference.is_empty());
    assert!(row.subscription_id.is_some());
    assert_eq!(data.subscriptions[0].config_count, 2);
    assert!(!data.subscriptions[0].reference.is_empty());
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(!body.contains("11111111-1111"));
    assert!(!body.contains("raw_config"));
    assert!(!body.contains("source_url"));
}
