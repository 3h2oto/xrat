use super::*;
use crate::app::tests::TestAppBuilder;

async fn read(
    service: &DiagnosticsService,
    cursor: Option<LogCursor>,
    limit: u16,
) -> (Vec<String>, LogCursor, bool) {
    match service
        .dispatch(DiagnosticsRequest::File {
            feed: LogFeed::Daemon,
            cursor,
            lines: limit,
        })
        .await
        .unwrap()
    {
        DiagnosticsPayload::File {
            lines,
            cursor,
            truncated,
        } => (lines, cursor, truncated),
        _ => panic!("expected log file"),
    }
}

#[tokio::test]
async fn bounded_follow_preserves_pages_partial_lines_and_truncation_recovery() {
    let (context, _root) = TestAppBuilder::new("diagnostics-file")
        .build_with_root()
        .await;
    tokio::fs::create_dir_all(&context.runtime_paths.runtime_dir)
        .await
        .unwrap();
    let path = context.runtime_paths.runtime_dir.join("daemon.log");
    let service = DiagnosticsService::new(context);
    let (empty, _, _) = read(&service, None, 2).await;
    assert!(empty.is_empty());
    tokio::fs::write(&path, b"old\nlast\n").await.unwrap();
    let (lines, cursor, _) = read(&service, None, 2).await;
    assert_eq!(lines, ["old", "last"]);
    tokio::fs::write(&path, b"old\nlast\none\ntwo\nthree\npartial")
        .await
        .unwrap();
    let (lines, cursor, _) = read(&service, Some(cursor), 2).await;
    assert_eq!(lines, ["one", "two"]);
    let (lines, cursor, _) = read(&service, Some(cursor), 2).await;
    assert_eq!(lines, ["three"]);
    let (lines, same, _) = read(&service, Some(cursor.clone()), 2).await;
    assert!(lines.is_empty());
    assert_eq!(cursor.offset, same.offset);
    tokio::fs::write(&path, b"old\nlast\none\ntwo\nthree\npartial done\n")
        .await
        .unwrap();
    let (lines, cursor, _) = read(&service, Some(cursor), 2).await;
    assert_eq!(lines, ["partial done"]);
    tokio::fs::write(&path, b"reset\n").await.unwrap();
    let (lines, _, _) = read(&service, Some(cursor), 2).await;
    assert_eq!(lines, ["reset"]);
    let huge = format!("{}\nend\n", "x".repeat(MAX_LOG_BYTES as usize + 20));
    tokio::fs::write(&path, huge).await.unwrap();
    let (lines, _, truncated) = read(&service, None, 2).await;
    assert_eq!(lines, ["end"]);
    assert!(truncated);
    let (lines, cursor, truncated) = read(
        &service,
        Some(LogCursor {
            session_id: None,
            offset: 0,
        }),
        2,
    )
    .await;
    assert!(truncated);
    assert_eq!(lines[0].len(), MAX_LOG_BYTES as usize);
    assert_eq!(cursor.offset, MAX_LOG_BYTES);
}

#[tokio::test]
async fn events_preserve_order_level_cursor_batch_limit_and_idempotent_clear() {
    let (context, _root) = TestAppBuilder::new("diagnostics-events")
        .build_with_root()
        .await;
    let mut ids = Vec::new();
    for level in ["info", "warn", "error", "info", "error"] {
        ids.push(
            context
                .db
                .record_event(&xrat_db::NewEvent {
                    level: level.into(),
                    source: "runtime".into(),
                    kind: "test".into(),
                    config_id: None,
                    session_id: None,
                    message: level.into(),
                    detail: Some("details".into()),
                })
                .await
                .unwrap(),
        );
    }
    let service = DiagnosticsService::new(context);
    let DiagnosticsPayload::Events(rows) = service
        .dispatch(DiagnosticsRequest::Events {
            after: None,
            limit: 2,
            level: None,
        })
        .await
        .unwrap()
    else {
        panic!("events");
    };
    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), ids[3..]);
    let DiagnosticsPayload::Events(rows) = service
        .dispatch(DiagnosticsRequest::Events {
            after: Some(0),
            limit: 2,
            level: Some(EventLevel::Warn),
        })
        .await
        .unwrap()
    else {
        panic!("events");
    };
    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), ids[1..3]);
    let DiagnosticsPayload::Events(rows) = service
        .dispatch(DiagnosticsRequest::Events {
            after: Some(ids[2]),
            limit: 2,
            level: Some(EventLevel::Warn),
        })
        .await
        .unwrap()
    else {
        panic!("events");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, ids[4]);
    for expected in [5, 0] {
        assert!(
            matches!(service.dispatch(DiagnosticsRequest::ClearEvents {}).await.unwrap(), DiagnosticsPayload::Cleared(count) if count == expected)
        );
    }
    for request in [
        DiagnosticsRequest::Events {
            after: Some(-1),
            limit: 1,
            level: None,
        },
        DiagnosticsRequest::Events {
            after: None,
            limit: 2001,
            level: None,
        },
        DiagnosticsRequest::File {
            feed: LogFeed::Daemon,
            cursor: None,
            lines: 0,
        },
    ] {
        assert!(service.dispatch(request).await.is_err());
    }
}

#[test]
fn diagnostics_wire_rejects_caller_paths_unknown_feeds_and_extra_fields() {
    for input in [
        r#"{"type":"file","feed":"daemon","lines":1,"path":"C:/private"}"#,
        r#"{"type":"file","feed":"../../private","lines":1}"#,
        r#"{"type":"clear_events","sql":"DELETE FROM configs"}"#,
        r#"{"type":"events","after":null,"limit":1,"level":"debug"}"#,
    ] {
        assert!(
            serde_json::from_str::<DiagnosticsRequest>(input).is_err(),
            "{input}"
        );
    }
}
