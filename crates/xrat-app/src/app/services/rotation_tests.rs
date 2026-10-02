use super::*;
use crate::app::tests::TestAppBuilder;
use crate::app::tests::fixtures::{test_node_with as test_node, test_source};
use std::time::{SystemTime, UNIX_EPOCH};
use xrat_db::RuntimeSessionInsert;

#[tokio::test]
async fn replacement_failure_persists_owner_metadata_and_event_without_ipc() {
    let context = TestAppBuilder::new("rotation-service-failure")
        .build()
        .await;
    context
        .db
        .import_nodes(&test_source(), &[test_node("example-a.com", "a")])
        .await
        .expect("node should import");
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should set");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be valid")
        .as_secs()
        .to_string();
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: xrat_db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some(now),
            stopped_at: None,
        })
        .await
        .expect("active session should insert");

    let failure = RotationService::new(&context, "service-test")
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(xrat_model::ConfigId(-1)),
        })
        .await
        .unwrap_err();
    assert_eq!(failure.reason, RotationFailureReason::CandidateFailed);
    assert!(failure.message.contains("config -1 was not found"));
    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("session should load")
        .expect("session should exist");
    assert_eq!(session.id, session_id);
    assert_eq!(
        session.last_transition_reason_code.as_deref(),
        Some("rotation_candidate_failed")
    );
    assert_eq!(session.owner_kind.as_deref(), Some("daemon"));
    assert_eq!(session.owner_instance_id.as_deref(), Some("service-test"));
    assert_eq!(
        session.last_transition_reason_detail.as_deref(),
        Some(failure.message.as_str())
    );
    let events = context
        .db
        .list_events(&xrat_db::EventFilter {
            limit: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    let event = events
        .iter()
        .find(|event| event.kind == "rotation_failed")
        .unwrap();
    assert_eq!(event.config_id, Some(xrat_model::ConfigId(-1)));
    assert_eq!(
        event.message,
        format!("Rotation failed (manual): {}", failure.message)
    );
}

#[tokio::test]
async fn failed_event_persistence_preserves_rotation_error() {
    let context = TestAppBuilder::new("rotation-service-event-failure")
        .build()
        .await;
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        context.runtime_paths.database_path.display()
    ))
    .await
    .unwrap();
    sqlx::query("DROP TABLE events")
        .execute(&pool)
        .await
        .unwrap();
    let failure = RotationService::new(&context, "service-test")
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Timer,
            candidate_id: Some(xrat_model::ConfigId(-1)),
        })
        .await
        .unwrap_err();
    assert_eq!(failure.reason, RotationFailureReason::CandidateFailed);
    assert!(failure.message.contains("config -1 was not found"));
}

#[tokio::test]
async fn no_candidate_returns_typed_reason_without_ipc() {
    let context = TestAppBuilder::new("rotation-service-no-candidate")
        .build()
        .await;
    let failure = RotationService::new(&context, "service-test")
        .replace(ReplaceRequest {
            trigger: RotationTrigger::HealthCheckFailed,
            candidate_id: None,
        })
        .await
        .unwrap_err();
    assert_eq!(failure.reason, RotationFailureReason::NoCandidate);
    assert_eq!(failure.reason.as_str(), "rotation_no_candidate");
}
