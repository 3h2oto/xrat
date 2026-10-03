use super::*;
use xrat_model::{Node, Protocol};

#[tokio::test]
async fn enable_on_deleted_config_is_noop_ok() {
    let context = test_context("enable-deleted").await;
    let id = seed_config(&context).await;
    context.db.delete_config(id).await.expect("soft delete");

    enable(&context, &EnableArgs { id: id.to_string() })
        .await
        .expect("noop ok");

    let config = fetch(&context, id).await;
    assert!(config.is_deleted);
}

#[tokio::test]
async fn enable_already_enabled_is_noop_ok() {
    let context = test_context("enable-already").await;
    let id = seed_config(&context).await;
    assert!(fetch(&context, id).await.is_enabled);

    enable(&context, &EnableArgs { id: id.to_string() })
        .await
        .expect("noop ok");
    assert!(fetch(&context, id).await.is_enabled);
}

#[tokio::test]
async fn disable_then_enable_round_trip() {
    let context = test_context("enable-roundtrip").await;
    let id = seed_config(&context).await;

    disable(&context, &DisableArgs { id: id.to_string() })
        .await
        .expect("disable ok");
    assert!(!fetch(&context, id).await.is_enabled);

    enable(&context, &EnableArgs { id: id.to_string() })
        .await
        .expect("enable ok");
    assert!(fetch(&context, id).await.is_enabled);
}

#[tokio::test]
async fn disable_already_disabled_is_noop_ok() {
    let context = test_context("disable-already").await;
    let id = seed_config(&context).await;
    disable(&context, &DisableArgs { id: id.to_string() })
        .await
        .expect("disable ok");

    disable(&context, &DisableArgs { id: id.to_string() })
        .await
        .expect("second disable noop ok");
    assert!(!fetch(&context, id).await.is_enabled);
}

async fn fetch(context: &AppContext, id: xrat_model::ConfigId) -> xrat_db::ConfigRecord {
    context
        .db
        .get_config_by_id(id)
        .await
        .expect("query should succeed")
        .expect("config should exist")
}

async fn seed_config(context: &AppContext) -> xrat_model::ConfigId {
    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::File,
        value: "seed.txt".to_string(),
        name: None,
    };
    context
        .db
        .import_nodes(&source, &[test_node("seed")])
        .await
        .expect("import should succeed");
    context
        .db
        .list_configs(&xrat_db::ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist")
        .id
}

fn test_node(name: &str) -> Node {
    Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("uuid-123".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: Some("/socket".to_string()),
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!("vless://uuid-123@example.com:443?type=ws#{name}"),
    }
}

async fn test_context(prefix: &str) -> AppContext {
    crate::app::tests::TestAppBuilder::new(prefix).build().await
}
