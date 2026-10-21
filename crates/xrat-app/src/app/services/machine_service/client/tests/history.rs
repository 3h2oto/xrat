use super::{fixture, import};
use crate::app::services::diagnostics::DiagnosticsRequest;
use xrat_db::{
    ConnectionTestCursor, ConnectionTestInsert, ConnectionTestRunInsert, ConnectionTestSelection,
};
use xrat_model::ConfigId;

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
        download_mbps: Some(1.5),
        upload_mbps: Some(0.5),
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        dial_endpoint_ip: Some("127.0.0.1".into()),
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
async fn probe_history_preserves_run_config_order_limits_and_missing_results() {
    let (context, client, calls, _root) = fixture(None).await;
    assert_eq!(client.latest_probe_run().await.unwrap(), None);
    import(&client, 1).await;
    import(&client, 2).await;
    let run = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "first".into(),
        })
        .await
        .unwrap();
    let other_run = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "second".into(),
        })
        .await
        .unwrap();
    for index in 0..5 {
        context
            .db
            .insert_connection_test(&measurement(
                ConfigId(1),
                if index == 0 { other_run } else { run },
            ))
            .await
            .unwrap();
    }
    context
        .db
        .insert_connection_test(&measurement(ConfigId(2), run))
        .await
        .unwrap();
    assert_eq!(
        client.latest_probe_run().await.unwrap(),
        context.db.get_latest_connection_test_run().await.unwrap()
    );
    let selection = ConnectionTestSelection::Config(ConfigId(1));
    let expected = context.db.list_connection_tests(ConfigId(1)).await.unwrap();
    assert_eq!(
        client.probe_history(selection, None).await.unwrap(),
        expected
    );
    assert_eq!(
        client.probe_history(selection, Some(3)).await.unwrap(),
        expected[..3]
    );
    assert_eq!(
        client
            .probe_history(ConnectionTestSelection::Run(run), None)
            .await
            .unwrap(),
        context.db.list_connection_tests_by_run(run).await.unwrap()
    );
    assert!(
        client
            .probe_history(ConnectionTestSelection::Run(i64::MAX), None)
            .await
            .unwrap()
            .is_empty()
    );
    let count = calls.lock().unwrap().len();
    assert!(
        client
            .probe_history(selection, Some(0))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(calls.lock().unwrap().len(), count);
    for (selection, before, limit) in [
        (ConnectionTestSelection::Config(ConfigId(0)), None, 1),
        (selection, None, 0),
        (
            selection,
            Some(ConnectionTestCursor {
                id: 0,
                tested_at: "date".into(),
            }),
            1,
        ),
        (
            selection,
            Some(ConnectionTestCursor {
                id: 1,
                tested_at: "".into(),
            }),
            1,
        ),
        (
            selection,
            Some(ConnectionTestCursor {
                id: 1,
                tested_at: "a".repeat(129),
            }),
            1,
        ),
    ] {
        assert!(
            client
                .diagnostics(DiagnosticsRequest::ProbePage {
                    selection,
                    before,
                    limit
                })
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn probe_history_does_not_return_partial_rows_when_a_later_read_fails() {
    let (context, client, _, _root) = fixture(Some(3)).await;
    import(&client, 1).await;
    let run = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "fixture".into(),
        })
        .await
        .unwrap();
    for _ in 0..5 {
        context
            .db
            .insert_connection_test(&measurement(ConfigId(1), run))
            .await
            .unwrap();
    }
    let error = client
        .probe_history(ConnectionTestSelection::Config(ConfigId(1)), None)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("pipe disconnected"));
}
