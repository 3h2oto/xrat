use super::*;
use crate::app::tests::{
    TestAppBuilder,
    fixtures::{test_node, test_source},
};

#[tokio::test]
async fn runtime_overview_preserves_shared_snapshot_and_live_supervisor_policy() {
    let (mut context, _root) = TestAppBuilder::new("runtime-overview")
        .build_with_root()
        .await;
    context.app_config.runtime.rotation.test_stages = vec!["tcp".into(), "real-delay".into()];
    context
        .db
        .import_nodes(&test_source(), &[test_node("overview.example")])
        .await
        .unwrap();
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .remove(0);
    let session_id = context
        .db
        .insert_runtime_session(&xrat_db::RuntimeSessionInsert {
            config_id: Some(config.id),
            status: xrat_db::RuntimeSessionStatus::Stopped,
            socks_host: Some("127.0.0.1".into()),
            socks_port: Some(1080),
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
    let mut state = SupervisorState::new("overview-owner".into());
    state.ready = false;
    state.rotation_enabled = true;
    state.rotation_interval_secs = 71;
    let (sender, receiver) = oneshot::channel();
    handle_runtime_overview(&state, &context, sender).await;
    let overview = receiver.await.unwrap().unwrap();
    assert!(!overview.daemon_ready);
    assert!(overview.rotation_enabled);
    assert_eq!(overview.rotation_interval_secs, 71);
    assert_eq!(overview.runtime.session.as_ref().unwrap().id, session_id);
    assert_eq!(overview.runtime.session_config.as_ref().unwrap(), &config);
    assert!(!overview.runtime.pid_running);
    assert_eq!(
        overview.config_path,
        context.runtime_paths.config_path.display().to_string()
    );
    assert_eq!(
        overview.test_stages,
        context.app_config.runtime.rotation.test_stages
    );
    let expected = RuntimeService::new(&context).status().await.unwrap();
    assert_eq!(
        serde_json::to_value(&overview.runtime).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    let wire: RuntimeOverview =
        serde_json::from_slice(&serde_json::to_vec(&overview).unwrap()).unwrap();
    assert_eq!(wire.runtime.session, overview.runtime.session);
    assert_eq!(wire.runtime.inbound_health, overview.runtime.inbound_health);
    assert_eq!(wire.runtime.status, overview.runtime.status);
    let (sender, receiver) = oneshot::channel();
    drop(receiver);
    handle_runtime_overview(&state, &context, sender).await;
}
