use super::*;
use crate::app::{config::ConnectionTestStage, tests::TestAppBuilder};

#[tokio::test]
async fn actual_runner_emits_queued_started_measured_and_finished_in_order() {
    let (mut context, _root) = TestAppBuilder::new("test-activity").build_with_root().await;
    context.app_config.testing.order = vec![ConnectionTestStage::Tcp];
    context.app_config.testing.geoip.enabled = false;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let nodes = xrat_config::parse_text(&format!(
        "vless://11111111-1111-1111-1111-111111111111@127.0.0.1:{port}#Activity"
    ));
    context
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::RawText,
                value: "activity-fixture".into(),
                name: None,
            },
            &nodes,
        )
        .await
        .unwrap();
    let id = context.db.list_configs(&Default::default()).await.unwrap()[0].id;
    let request = super::super::TestRunRequest {
        skip_icmp: true,
        skip_real_delay: true,
        skip_download: true,
        skip_upload: true,
        skip_tcp: false,
        concurrency: Some(1),
        ..Default::default()
    };
    let flag = xrat_support::cancel::CancellationFlag::new();
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
    let (activity_tx, mut activity_rx) = tokio::sync::mpsc::unbounded_channel();
    let count = super::super::run_bulk_for_config_ids_with_activity(
        &request,
        &context,
        &[id],
        xrat_support::cancel::CancellationReceiver::from_flag(&flag),
        progress_tx,
        activity_tx,
    )
    .await
    .unwrap();
    assert_eq!(count, 1);
    assert!(
        matches!(activity_rx.recv().await, Some(TestActivity::Queued { config_id, .. }) if config_id == id)
    );
    assert!(
        matches!(activity_rx.recv().await, Some(TestActivity::Started { config_id, metric: TestMetric::Tcp }) if config_id == id)
    );
    assert!(
        matches!(activity_rx.recv().await, Some(TestActivity::Measured { config_id, metric: TestMetric::Tcp, passed: false }) if config_id == id)
    );
    assert!(
        matches!(activity_rx.recv().await, Some(TestActivity::Finished { config_id, failed: false }) if config_id == id)
    );
    assert!(activity_rx.recv().await.is_none());
    let update = progress_rx.recv().await.unwrap();
    assert_eq!((update.config_id, update.done, update.total), (id, 1, 1));
    assert!(progress_rx.recv().await.is_none());
}
