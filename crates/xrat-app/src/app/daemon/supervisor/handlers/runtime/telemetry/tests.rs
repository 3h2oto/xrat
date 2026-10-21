use super::*;
use crate::app::tests::{TestAppBuilder, fixtures::test_node};
use xrat_web::types::LatencyMetric;

#[tokio::test]
async fn runtime_telemetry_reads_owner_policy_and_store_without_exposing_settings() {
    let (context, _root) = TestAppBuilder::new("runtime-telemetry-owner")
        .build_with_root()
        .await;
    std::fs::create_dir_all(context.runtime_paths.config_path.parent().unwrap()).unwrap();
    std::fs::write(&context.runtime_paths.config_path,
        "[testing.tcp]\nenabled = false\n[testing.icmp]\nenabled = true\n[testing.real_delay]\nenabled = false\n[runtime.stats]\nenabled = false\n").unwrap();
    context
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::Url,
                value: "https://owner.example/subscription".into(),
                name: None,
            },
            &[test_node("owner.example")],
        )
        .await
        .unwrap();
    let (sender, receiver) = oneshot::channel();
    handle_runtime_dashboard(&context, sender).await;
    let dashboard = receiver.await.unwrap().unwrap();
    assert_eq!(dashboard.enabled_metrics, vec![LatencyMetric::Icmp]);
    assert!(!dashboard.stats_enabled);
    assert_eq!(dashboard.runtime_status, "stopped");
    assert!(dashboard.session_id.is_none());
    assert!(dashboard.probes.is_empty());
    assert!(dashboard.last_subscription_refresh.is_some());
    assert!(dashboard.traffic_history.is_empty());
    let encoded = serde_json::to_string(&dashboard).unwrap();
    for private in ["owner.example", "config_path", "password", "database"] {
        assert!(!encoded.contains(private));
    }
    let (sender, receiver) = oneshot::channel();
    handle_runtime_traffic(&context, sender).await;
    assert_eq!(
        receiver.await.unwrap().unwrap(),
        TrafficSnapshot {
            session_id: None,
            traffic: None
        }
    );
    let (sender, receiver) = oneshot::channel();
    drop(receiver);
    handle_runtime_dashboard(&context, sender).await;
    let (sender, receiver) = oneshot::channel();
    drop(receiver);
    handle_runtime_traffic(&context, sender).await;
    std::fs::write(&context.runtime_paths.config_path, "invalid = [").unwrap();
    let (sender, receiver) = oneshot::channel();
    handle_runtime_dashboard(&context, sender).await;
    assert!(receiver.await.unwrap().is_err());
    let (sender, receiver) = oneshot::channel();
    handle_runtime_traffic(&context, sender).await;
    assert!(receiver.await.unwrap().is_err());
}
