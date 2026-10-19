use super::*;
use crate::{ConnectionTestInsert, RuntimeSessionInsert, RuntimeSessionStatus};

fn node(name: &str) -> Node {
    let mut node = test_node(name);
    node.address = format!("purge-{name}.example.test");
    node
}

#[tokio::test]
async fn refresh_purges_backlog_and_history_but_protects_runtime_and_other_sources() {
    let path = test_database_path("refresh-auto-purge");
    let db = Database::connect_sqlite(&path).await.unwrap();
    verify_refresh_purge(&db).await;
    let _ = std::fs::remove_file(path);
}

pub(in crate::database::tests) async fn verify_refresh_purge(db: &Database) {
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.test/purge-backlog".into(),
        name: None,
    };
    let names = [
        "keep", "drop", "legacy", "starting", "running", "stopping", "active",
    ];
    let nodes: Vec<_> = names.iter().map(|name| node(name)).collect();
    let imported = db.import_nodes(&source, &nodes).await.unwrap();
    let filter = ConfigListFilter {
        subscription_id: Some(imported.subscription_id),
        include_deleted: true,
        ..Default::default()
    };
    let rows = db.list_configs(&filter).await.unwrap();
    let id = |name: &str| {
        rows.iter()
            .find(|row| row.address == format!("purge-{name}.example.test"))
            .unwrap()
            .id
    };
    db.delete_config(id("legacy")).await.unwrap();
    db.set_active_config(id("active")).await.unwrap();
    let mut guarded = Vec::new();
    for (name, status) in [
        ("starting", RuntimeSessionStatus::Starting),
        ("running", RuntimeSessionStatus::Running),
        ("stopping", RuntimeSessionStatus::Stopping),
        ("drop", RuntimeSessionStatus::Stopped),
    ] {
        let session = db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(id(name)),
                status,
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
        if name != "drop" {
            guarded.push(session);
        }
    }
    db.insert_connection_test(&ConnectionTestInsert {
        config_id: id("drop"),
        run_id: None,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: Some(true),
        tcp_ms: Some(10),
        real_delay_ok: None,
        real_delay_ms: None,
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
        failure_kind: None,
        failure_reason: None,
    })
    .await
    .unwrap();
    let other = ImportSource {
        kind: SourceKind::RawText,
        value: "other-purge-source".into(),
        name: None,
    };
    let other_import = db.import_nodes(&other, &[node("unrelated")]).await.unwrap();
    let other_id = db
        .list_configs(&ConfigListFilter {
            subscription_id: Some(other_import.subscription_id),
            ..Default::default()
        })
        .await
        .unwrap()[0]
        .id;
    db.delete_config(other_id).await.unwrap();
    db.import_nodes(&source, &[node("keep")]).await.unwrap();
    for name in ["drop", "legacy"] {
        assert!(db.get_config_by_id(id(name)).await.unwrap().is_none());
    }
    assert!(
        db.recent_connection_tests(id("drop"), 10)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        db.get_latest_runtime_session_for_config(id("drop"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        db.get_config_by_id(other_id)
            .await
            .unwrap()
            .unwrap()
            .is_deleted
    );
    assert_eq!(
        db.get_config_by_id(id("keep"))
            .await
            .unwrap()
            .unwrap()
            .r#ref,
        rows.iter().find(|row| row.id == id("keep")).unwrap().r#ref
    );
    for name in ["starting", "running", "stopping", "active"] {
        assert!(
            db.get_config_by_id(id(name))
                .await
                .unwrap()
                .unwrap()
                .is_deleted
        );
    }
    for name in ["starting", "running", "stopping"] {
        assert!(
            db.get_latest_runtime_session_for_config(id(name))
                .await
                .unwrap()
                .is_some()
        );
    }
    db.import_nodes(&source, &[]).await.unwrap();
    assert_eq!(db.list_configs(&filter).await.unwrap().len(), 5);
    for session in guarded {
        db.mark_runtime_session_stopped(session, None)
            .await
            .unwrap();
    }
    db.clear_active_config().await.unwrap();
    db.import_nodes(&source, &[node("keep")]).await.unwrap();
    assert_eq!(db.list_configs(&filter).await.unwrap().len(), 1);
    assert!(db.get_config_by_id(other_id).await.unwrap().is_some());
}

#[tokio::test]
async fn cleanup_failure_rolls_back_import_and_deletion_flags() {
    let path = test_database_path("refresh-purge-rollback");
    let db = Database::connect_sqlite(&path).await.unwrap();
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.test/purge-rollback".into(),
        name: None,
    };
    let imported = db
        .import_nodes(&source, &[node("before-a"), node("before-b")])
        .await
        .unwrap();
    let crate::connection::DbPool::Sqlite(pool) = &db.pool else {
        unreachable!()
    };
    sqlx::query("CREATE TRIGGER fail_refresh_purge BEFORE DELETE ON configs BEGIN SELECT RAISE(ABORT, 'cleanup failure'); END").execute(pool).await.unwrap();
    assert!(
        db.import_nodes(&source, &[node("before-a"), node("new-c")])
            .await
            .is_err()
    );
    let rows = db
        .list_configs(&ConfigListFilter {
            subscription_id: Some(imported.subscription_id),
            include_deleted: true,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| !row.is_deleted));
    assert!(
        rows.iter()
            .all(|row| row.address != "purge-new-c.example.test")
    );
    let _ = std::fs::remove_file(path);
}
