use super::*;
use crate::app::tests::{
    TestAppBuilder,
    fixtures::{test_node, test_source},
};
use xrat_db::{RuntimeSessionInsert, RuntimeSessionStatus};

#[test]
fn machine_configuration_rejects_database_and_engine_path_overrides() {
    assert!(validate_config(&AppConfig::default()).is_ok());
    for text in [
        "[paths]\nxray='C:\\Users\\operator\\xray.exe'",
        "[paths]\nsing_box='C:\\Users\\operator\\sing-box.exe'",
        "[paths]\nv2ray='C:\\Users\\operator\\v2ray.exe'",
        "[database.sqlite]\npath='elsewhere.sqlite'",
        "[paths]\ndatabase='elsewhere.sqlite'",
        "[database]\nbackend='postgres'",
    ] {
        let config: AppConfig = toml::from_str(text).unwrap();
        assert!(validate_config(&config).is_err(), "{text}");
    }
}

#[tokio::test]
async fn machine_startup_reconciles_stale_session_without_reconnecting_or_starting_rotation() {
    let (mut context, _root) = TestAppBuilder::new("machine-startup")
        .build_with_root()
        .await;
    context.app_config.server.enabled = true;
    context.app_config.runtime.rotation.enabled = true;
    context.app_config.runtime.rotation.refresh_subscriptions = true;
    context.app_config.subscriptions.auto_refresh = true;
    context
        .db
        .import_nodes(&test_source(), &[test_node("example.com")])
        .await
        .unwrap();
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .remove(0);
    context.db.set_active_config(config.id).await.unwrap();
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Running,
            socks_host: None,
            socks_port: None,
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: None,
            started_at: Some("1".into()),
            stopped_at: None,
        })
        .await
        .unwrap();
    for _ in 0..2 {
        prepare_runtime(&mut context).await.unwrap();
        let session = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            session.id, session_id,
            "startup must not create a replacement session"
        );
        assert_eq!(session.status, RuntimeSessionStatus::Failed);
        assert!(context.db.get_active_config().await.unwrap().is_none());
        assert!(!context.app_config.runtime.rotation.enabled);
        assert!(!context.app_config.runtime.rotation.refresh_subscriptions);
        assert!(!context.app_config.server.enabled);
        assert!(!context.app_config.subscriptions.auto_refresh);
    }
}
