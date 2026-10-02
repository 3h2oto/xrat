use super::*;
use crate::app::services::test_support::FakeClock;
use crate::app::tests::{
    TestAppBuilder,
    fixtures::{test_node, test_source},
};

#[tokio::test]
async fn overview_preserves_deleted_visibility_and_fresh_empty_geo_cache() {
    let (mut context, _root) = TestAppBuilder::new("dashboard-cache")
        .build_with_root()
        .await;
    context.app_config.server.host = "127.0.0.1".into();
    let nodes = [
        test_node("fresh.example"),
        test_node("empty.example"),
        test_node("stale.example"),
        test_node("deleted.example"),
    ];
    context
        .db
        .import_nodes(&test_source(), &nodes)
        .await
        .unwrap();
    let rows = context.db.list_configs(&Default::default()).await.unwrap();
    context.db.delete_config(rows[3].id).await.unwrap();
    for (host, country, resolved_at) in [
        ("fresh.example", Some("NL"), 99999),
        ("empty.example", None, 99999),
        ("stale.example", Some("DE"), 1),
    ] {
        context
            .db
            .upsert_geoip_cache(&xrat_db::GeoIpCacheUpsert {
                host: host.into(),
                ip: None,
                country: country.map(str::to_string),
                location: None,
                asn: None,
                resolved_at,
            })
            .await
            .unwrap();
    }
    let service = DashboardService::with_clock(&context, FakeClock::at(100000));
    let snapshot = service.load(false).await.unwrap();
    assert_eq!(snapshot.configs.len(), 3);
    assert_eq!(snapshot.sources.len(), 1);
    assert_eq!(
        snapshot.configs[0].endpoint_location.country.as_deref(),
        Some("NL")
    );
    assert_eq!(
        snapshot.pending_enrichment,
        vec![(rows[2].id, "stale.example".into())]
    );
    assert!(!snapshot.daemon.running);
    assert!(snapshot.probe_history.is_empty());
    assert_eq!(
        snapshot.api_b64_url,
        format!("http://127.0.0.1:{}/b64", context.app_config.server.port)
    );
    let snapshot = service.load(true).await.unwrap();
    assert_eq!(snapshot.configs.len(), 4);
    assert!(snapshot.configs[3].summary.is_deleted);
    assert_eq!(snapshot.pending_enrichment.len(), 2);
}

#[tokio::test]
async fn log_service_keeps_latest_session_tail_and_event_limits() {
    let (context, _root) = TestAppBuilder::new("dashboard-logs")
        .build_with_root()
        .await;
    let session_id = context
        .db
        .insert_runtime_session(&xrat_db::RuntimeSessionInsert {
            config_id: None,
            status: xrat_db::RuntimeSessionStatus::Stopped,
            socks_host: None,
            socks_port: None,
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: None,
            started_at: None,
            stopped_at: None,
        })
        .await
        .unwrap();
    std::fs::create_dir_all(&context.runtime_paths.runtime_dir).unwrap();
    std::fs::write(
        context
            .runtime_paths
            .runtime_dir
            .join(format!("session-{session_id}.out.log")),
        (0..205)
            .map(|index| format!("row-{index}\r\n"))
            .collect::<String>(),
    )
    .unwrap();
    for index in 0..205 {
        context
            .db
            .record_event(&xrat_db::NewEvent {
                level: "info".into(),
                source: "runtime".into(),
                kind: "test".into(),
                config_id: None,
                session_id: Some(session_id),
                message: format!("event-{index}"),
                detail: None,
            })
            .await
            .unwrap();
    }
    let logs = DashboardLogs::load(&context).await.unwrap();
    assert_eq!(logs.events.len(), 200);
    assert_eq!(logs.proxy.len(), 200);
    assert_eq!(logs.proxy.first().unwrap().message, "row-5");
    assert_eq!(logs.proxy.last().unwrap().message, "row-204");
}
