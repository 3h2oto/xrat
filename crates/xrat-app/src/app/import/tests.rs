use super::{load_single_node, persist_nodes};
use xrat_db::{Database, ImportSource, SourceKind};
use xrat_model::Protocol;

#[test]
fn parses_single_manual_config_as_raw_text_source() {
    let (source, node) =
        load_single_node("vless://uuid-123@example.com:443?type=ws&security=tls#Example%20Node")
            .expect("single config should parse");

    assert_eq!(source.kind, SourceKind::RawText);
    assert_eq!(node.protocol, Protocol::Vless);
    assert_eq!(node.address, "example.com");
}

#[test]
fn rejects_multiple_configs_for_add() {
    let err = load_single_node(
        "vless://uuid-123@example.com:443#One\nss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#Two",
    )
    .expect_err("multiple configs should fail");

    assert!(err.to_string().contains("exactly one"));
}

#[tokio::test]
async fn supplied_name_is_persisted_and_updates_existing_url() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let database = Database::connect(&xrat_db::DatabaseConnectionConfig::Sqlite {
        path: root.path().join("db.sqlite"),
    })
    .await
    .expect("database should connect");

    for name in ["First", "Second"] {
        let (_, node) =
            load_single_node("vless://uuid-123@example.com:443#One").expect("config should parse");
        let source = ImportSource {
            kind: SourceKind::Url,
            value: "https://example.com/sub".to_string(),
            name: None,
        };
        persist_nodes(&database, source, &[node], Some(name))
            .await
            .expect("import should persist");
    }

    let subscriptions = database
        .list_subscriptions()
        .await
        .expect("subscriptions should load");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].name.as_deref(), Some("Second"));
}

#[tokio::test]
async fn omitted_name_preserves_existing_import_behavior() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let database = Database::connect(&xrat_db::DatabaseConnectionConfig::Sqlite {
        path: root.path().join("db.sqlite"),
    })
    .await
    .expect("database should connect");
    let (_, node) =
        load_single_node("vless://uuid-123@example.com:443#One").expect("config should parse");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: None,
    };

    persist_nodes(&database, source, &[node], None)
        .await
        .expect("import should persist");

    let subscription = database
        .list_subscriptions()
        .await
        .expect("subscriptions should load")
        .pop()
        .expect("subscription should exist");
    assert_eq!(subscription.name, None);
}
