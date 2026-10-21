use super::*;
use xrat_web::types::{LatencyMetric, Overview};

#[tokio::test]
async fn dashboard_reports_fresh_settings_empty_states_and_real_refresh_times() {
    let (router, state) = fixture().await;
    let context = state.web.as_ref().unwrap().context.as_ref().unwrap();
    std::fs::create_dir_all(context.runtime_paths.config_path.parent().unwrap()).unwrap();
    std::fs::write(&context.runtime_paths.config_path, "[testing.tcp]\nenabled = true\n[testing.icmp]\nenabled = false\n[testing.real_delay]\nenabled = false\n[runtime.stats]\nenabled = false\n[runtime.tun]\nenabled = true\n").unwrap();
    let (cookie, _) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/overview?dashboard=true",
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
    let meta = data.dashboard.unwrap();
    assert!(meta.sampled_at_ms.is_finite() && meta.sampled_at_ms > 0.0);
    assert_eq!(meta.enabled_metrics, vec![LatencyMetric::Tcp]);
    assert!(!meta.stats_enabled);
    assert!(meta.tun_enabled);
    assert!(!meta.tun_active);
    assert!(meta.session_id.is_none());
    assert!(meta.probes.is_empty());
    assert!(meta.last_subscription_refresh.is_none());
    assert!(data.traffic.is_none());
    assert!(!data.running);
    assert!(
        !String::from_utf8(bytes.to_vec())
            .unwrap()
            .contains("password_hash")
    );

    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::Url,
        value: "https://example.test/sub".into(),
        name: None,
    };
    let nodes = xrat_config::parse_text(
        "vless://11111111-1111-1111-1111-111111111111@one.example:443?security=tls#One",
    );
    state.db.import_nodes(&source, &nodes).await.unwrap();
    let before = super::super::security::now();
    let last = state
        .db
        .last_subscription_refresh(before)
        .await
        .unwrap()
        .unwrap();
    assert!((before - 2..=before).contains(&last));
    let subscription = state.db.list_subscriptions().await.unwrap().remove(0);
    state
        .db
        .set_subscription_name(subscription.id, "Renamed")
        .await
        .unwrap();
    assert_eq!(
        state.db.last_subscription_refresh(before).await.unwrap(),
        Some(last)
    );
    assert!(
        state
            .db
            .last_subscription_refresh(last - 1)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        request(
            &router,
            "/api/v1/overview?dashboard=true",
            "GET",
            serde_json::Value::Null,
            "",
            "",
            ""
        )
        .await
        .status()
            == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn dashboard_probe_window_is_bounded_and_hides_failed_or_negative_measurements() {
    let (_, state) = fixture().await;
    let context = state.web.as_ref().unwrap().context.as_ref().unwrap();
    let nodes = xrat_config::parse_text(
        "vless://11111111-1111-1111-1111-111111111111@one.example:443?security=tls#One",
    );
    state
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::RawText,
                value: "fixture".into(),
                name: None,
            },
            &nodes,
        )
        .await
        .unwrap();
    let config = state
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .remove(0);
    let mut latest_id = 0;
    for _ in 0..35 {
        latest_id = state
            .db
            .insert_connection_test(&xrat_db::ConnectionTestInsert {
                run_id: None,
                config_id: config.id,
                icmp_ok: Some(false),
                icmp_ms: Some(123),
                tcp_ok: Some(true),
                tcp_ms: Some(0),
                real_delay_ok: Some(true),
                real_delay_ms: Some(-1),
                download_mbps: None,
                upload_mbps: None,
                connect_ms: None,
                ttfb_ms: None,
                http_status: None,
                dial_endpoint_ip: None,
                dial_endpoint_location: None,
                dial_endpoint_country: None,
                dial_endpoint_asn: None,
                dial_endpoint_geoip_source: None,
                dial_endpoint_fronting: None,
                failure_kind: Some("timeout".into()),
                failure_reason: Some("private diagnostic".into()),
            })
            .await
            .unwrap();
    }
    let mut snapshot = crate::app::runtime_service::RuntimeService::new(context)
        .status()
        .await
        .unwrap();
    snapshot.pid_running = true;
    snapshot.active_config = Some(config);
    let meta = super::super::dashboard::load(context, &snapshot)
        .await
        .unwrap();
    assert_eq!(meta.probes.len(), 30);
    assert_eq!(meta.probes.last().unwrap().id, latest_id);
    assert!(meta.probes.windows(2).all(|pair| pair[0].id < pair[1].id));
    for point in meta.probes {
        assert_eq!(point.icmp_ms, None);
        assert_eq!(point.tcp_ms, Some(0));
        assert_eq!(point.real_delay_ms, None);
    }
}

#[tokio::test]
async fn runtime_details_require_auth_and_handle_no_daemon_or_connection() {
    let (router, state) = fixture().await;
    let context = state.web.as_ref().unwrap().context.as_ref().unwrap();
    std::fs::create_dir_all(context.runtime_paths.config_path.parent().unwrap()).unwrap();
    std::fs::write(
        &context.runtime_paths.config_path,
        "[runtime]\nengine = 'sing-box'\n[runtime.rotation]\nenabled = true\ninterval_secs = 600\n",
    )
    .unwrap();
    assert_eq!(
        request(
            &router,
            "/api/v1/overview/runtime",
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
    let (cookie, _) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/overview/runtime",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let runtime: xrat_web::types::RuntimeDetails = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(runtime.engine, "sing-box");
    assert_eq!(runtime.status, "stopped");
    assert_eq!(runtime.daemon_running, Some(false));
    assert!(runtime.process_id.is_none());
    assert!(runtime.started_at.is_none());
    assert!(runtime.inbounds.is_empty());
    assert!(runtime.rotation_enabled);
    assert_eq!(runtime.rotation_interval_secs, 600);
    assert_eq!(runtime.engines.len(), 2);
    assert_eq!(runtime.running_jobs, 0);
    assert_eq!(runtime.queued_jobs, 0);
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(!body.contains("password"));
    assert!(!body.contains("database"));
    assert!(!body.contains("config_path"));
}
