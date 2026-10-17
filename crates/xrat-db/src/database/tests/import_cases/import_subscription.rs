use super::*;
use std::collections::BTreeMap;

#[tokio::test]
async fn subscription_headers_are_preserved_or_replaced_on_import() {
    let db_path = test_database_path("xrat-subscription-headers");
    let db = Database::connect_sqlite(&db_path).await.unwrap();
    verify_subscription_headers(&db).await;
    let _ = std::fs::remove_file(db_path);
}

pub(in super::super) async fn verify_subscription_headers(db: &Database) {
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/headers".into(),
        name: None,
    };
    let mut node = test_node("headers");
    node.address = "headers.example.com".into();
    node.raw_config = "vless://uuid-123@headers.example.com:443#headers".into();
    let original = BTreeMap::from([
        ("x-hwid".into(), "device-secret".into()),
        ("user-agent".into(), "Happ".into()),
    ]);
    let replacement = BTreeMap::from([("x-hwid".into(), "replacement-secret".into())]);
    let empty = BTreeMap::new();
    for (supplied, expected) in [
        (Some(&original), &original),
        (None, &original),
        (Some(&replacement), &replacement),
        (Some(&empty), &empty),
    ] {
        let summary = db
            .import_nodes_with_headers(&source, std::slice::from_ref(&node), supplied)
            .await
            .unwrap();
        let stored = db
            .get_subscription_by_id(summary.subscription_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&stored.headers, expected);
        assert_eq!(
            db.list_subscriptions()
                .await
                .unwrap()
                .iter()
                .find(|subscription| subscription.id == stored.id)
                .unwrap()
                .headers,
            *expected
        );
        assert_eq!(
            db.list_refreshable_due_subscriptions(i64::MAX)
                .await
                .unwrap()
                .iter()
                .find(|subscription| subscription.id == stored.id)
                .unwrap()
                .headers,
            *expected
        );
    }
}

#[tokio::test]
async fn imports_nodes_and_creates_subscription() {
    let db_path = test_database_path("xrat-import");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: Some("Example".to_string()),
    };

    let summary = db
        .import_nodes(&source, &[test_node("first")])
        .await
        .expect("import should succeed");

    assert_eq!(summary.imported_configs, 1);
    assert_eq!(summary.total_configs, 1);
    assert_eq!(db.get_subscription_count().await.expect("count"), 1);
    assert_eq!(db.get_config_count().await.expect("count"), 1);
    assert_eq!(db.get_connection_test_count().await.expect("count"), 0);
    assert_eq!(db.get_runtime_session_count().await.expect("count"), 0);

    let subscriptions = db.list_subscriptions().await.expect("subscriptions");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].source_kind, "url");
    assert_eq!(subscriptions[0].config_count, 1);

    let id = subscriptions[0].id;
    let fetched = db
        .get_subscription_by_id(id)
        .await
        .expect("query should succeed")
        .expect("subscription should exist");
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.config_count, 1);
    assert_eq!(fetched.name.as_deref(), Some("Example"));
    assert!(
        db.get_subscription_by_id(SubscriptionId(9999))
            .await
            .expect("query should succeed")
            .is_none()
    );

    let _ = std::fs::remove_file(db_path);
}
