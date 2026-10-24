use super::super::import_cases::test_node;
use super::super::*;
use crate::{ConnectionTestCursor, ConnectionTestSelection};

fn measurement(config_id: ConfigId, run_id: i64) -> ConnectionTestInsert {
    ConnectionTestInsert {
        config_id,
        run_id: Some(run_id),
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
    }
}

#[tokio::test]
async fn history_cursor_preserves_timestamp_order_ties_selection_and_append_boundaries() {
    let path = test_database_path("connection-history-cursor");
    let db = Database::connect_sqlite(&path).await.unwrap();
    let mut second = test_node("second");
    second.address = "other.example".into();
    db.import_nodes(
        &ImportSource {
            kind: SourceKind::RawText,
            value: "cursor fixture".into(),
            name: None,
        },
        &[test_node("first"), second],
    )
    .await
    .unwrap();
    let configs = db.list_configs(&Default::default()).await.unwrap();
    assert_eq!(configs.len(), 2);
    let first = configs[0].id;
    let second = configs[1].id;
    let run = db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "first".into(),
        })
        .await
        .unwrap();
    let other_run = db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "second".into(),
        })
        .await
        .unwrap();
    let mut ids = Vec::new();
    for _ in 0..3 {
        ids.push(
            db.insert_connection_test(&measurement(first, run))
                .await
                .unwrap(),
        );
    }
    db.insert_connection_test(&measurement(second, other_run))
        .await
        .unwrap();
    db.insert_connection_test(&measurement(first, other_run))
        .await
        .unwrap();
    let crate::connection::DbPool::Sqlite(pool) = &db.pool else {
        panic!("sqlite");
    };
    sqlx::query("UPDATE connection_tests SET tested_at = ? WHERE id = ?")
        .bind("2000-01-01 00:00:00")
        .bind(ids[1])
        .execute(pool)
        .await
        .unwrap();
    let expected = db.list_connection_tests(first).await.unwrap();
    assert_eq!(expected.last().unwrap().id, ids[1]);
    let selection = ConnectionTestSelection::Config(first);
    let mut actual = db.connection_test_page(&selection, None, 2).await.unwrap();
    assert_eq!(actual.len(), 2);
    let new_id = db
        .insert_connection_test(&measurement(first, run))
        .await
        .unwrap();
    loop {
        let last = actual.last().unwrap();
        let cursor = ConnectionTestCursor {
            id: last.id,
            tested_at: last.tested_at.clone(),
        };
        let page = db
            .connection_test_page(&selection, Some(&cursor), 2)
            .await
            .unwrap();
        if page.is_empty() {
            break;
        }
        actual.extend(page);
    }
    assert_eq!(actual, expected);
    assert_eq!(
        db.connection_test_page(&selection, None, 1).await.unwrap()[0].id,
        new_id
    );
    let by_run = db
        .connection_test_page(&ConnectionTestSelection::Run(run), None, 200)
        .await
        .unwrap();
    assert_eq!(by_run, db.list_connection_tests_by_run(run).await.unwrap());
    let cursor = ConnectionTestCursor {
        id: i64::MAX,
        tested_at: "' OR 1=1 --".into(),
    };
    assert!(
        db.connection_test_page(&ConnectionTestSelection::Config(second), Some(&cursor), 1)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        db.connection_test_page(&selection, None, 0)
            .await
            .unwrap()
            .len(),
        1
    );
    let _ = std::fs::remove_file(path);
}
