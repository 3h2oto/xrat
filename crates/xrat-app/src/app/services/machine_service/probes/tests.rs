use super::*;
use crate::app::config::ConnectionTestStage;
use crate::app::tests::TestAppBuilder;
use xrat_model::ConfigId;

async fn terminal(jobs: &ProbeJobs, id: &str) -> ProbeSnapshot {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let snapshot = jobs
                .dispatch(ProbeRequest::Status {
                    job_id: Some(id.into()),
                })
                .await
                .unwrap()
                .unwrap();
            if !matches!(snapshot.state, ProbeState::Running | ProbeState::Cancelling) {
                return snapshot;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn machine_probes_use_shared_results_and_exact_run_identity() {
    let (mut context, _root) = TestAppBuilder::new("machine-probes")
        .build_with_root()
        .await;
    context.app_config.testing.order = vec![ConnectionTestStage::Tcp];
    context.app_config.testing.geoip.enabled = false;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let nodes = xrat_config::parse_text(&format!(
        "vless://11111111-1111-1111-1111-111111111111@127.0.0.1:{}#probe",
        listener.local_addr().unwrap().port()
    ));
    context
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::RawText,
                value: "machine-probe-fixture".into(),
                name: None,
            },
            &nodes,
        )
        .await
        .unwrap();
    let id = context.db.list_configs(&Default::default()).await.unwrap()[0].id;
    let jobs = ProbeJobs::new(context.clone());
    assert!(
        jobs.dispatch(ProbeRequest::Status { job_id: None })
            .await
            .unwrap()
            .is_none()
    );
    let start = jobs
        .dispatch(ProbeRequest::Start {
            config_ids: vec![id, ConfigId(i64::MAX)],
            stages: ProbeStages::default(),
        })
        .await
        .unwrap()
        .unwrap();
    let finished = terminal(&jobs, &start.job_id).await;
    assert_eq!(finished.state, ProbeState::Completed);
    assert_eq!((finished.completed, finished.total), (1, 1));
    assert_eq!(finished.last_config_id, Some(id));
    let run_id = finished.run_id.unwrap();
    let page = jobs
        .progress(&finished.job_id, 0, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.rows[0].config_id, id.0);
    assert!(
        page.rows[0]
            .metrics
            .iter()
            .any(|metric| metric.metric == xrat_web::types::TestMetric::Tcp
                && metric.state == xrat_web::types::TestMetricState::Passed)
    );
    for (offset, limit) in [(0, 0), (0, 201), (10_001, 1)] {
        assert!(
            jobs.progress(&finished.job_id, offset, limit)
                .await
                .is_err()
        );
    }
    let rows = context
        .db
        .list_connection_tests_by_run(run_id)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].config_id, id);
    assert_eq!(rows[0].tcp_ok, Some(true));
    let latest = context
        .db
        .get_latest_connection_test_run()
        .await
        .unwrap()
        .unwrap();
    assert_eq!((latest.id, latest.kind.as_str()), (run_id, "machine"));
    let again = jobs
        .dispatch(ProbeRequest::Cancel {
            job_id: start.job_id,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(again, finished);
    jobs.shutdown().await.unwrap();
}

#[tokio::test]
async fn machine_probes_reject_bad_batches_and_shutdown_cancels_without_restarting() {
    let (context, _root) = TestAppBuilder::new("machine-probe-cancel")
        .build_with_root()
        .await;
    let jobs = ProbeJobs::new(context.clone());
    for ids in [
        vec![],
        vec![ConfigId(0)],
        vec![ConfigId(-1)],
        vec![ConfigId(1); 2],
        vec![ConfigId(1); 10_001],
    ] {
        assert!(
            jobs.dispatch(ProbeRequest::Start {
                config_ids: ids,
                stages: Default::default()
            })
            .await
            .is_err()
        );
    }
    assert!(
        context
            .db
            .get_latest_connection_test_run()
            .await
            .unwrap()
            .is_none()
    );
    let start = jobs
        .dispatch(ProbeRequest::Start {
            config_ids: vec![ConfigId(1)],
            stages: Default::default(),
        })
        .await
        .unwrap()
        .unwrap();
    assert!(
        jobs.dispatch(ProbeRequest::Start {
            config_ids: vec![ConfigId(2)],
            stages: Default::default()
        })
        .await
        .is_err()
    );
    let cancelling = jobs
        .dispatch(ProbeRequest::Cancel {
            job_id: start.job_id.clone(),
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cancelling.state, ProbeState::Cancelling);
    jobs.shutdown().await.unwrap();
    let cancelled = terminal(&jobs, &start.job_id).await;
    assert_eq!(cancelled.state, ProbeState::Cancelled);
    assert_eq!(cancelled.completed, 0);
    assert!(
        jobs.dispatch(ProbeRequest::Start {
            config_ids: vec![ConfigId(3)],
            stages: Default::default()
        })
        .await
        .is_err()
    );
    jobs.shutdown().await.unwrap();
    assert!(
        jobs.dispatch(ProbeRequest::Status {
            job_id: Some("arbitrary/path".into())
        })
        .await
        .is_err()
    );
    assert!(
        jobs.dispatch(ProbeRequest::Cancel {
            job_id: "arbitrary/path".into()
        })
        .await
        .is_err()
    );
    assert!(
        jobs.dispatch(ProbeRequest::Cancel {
            job_id: uuid::Uuid::new_v4().to_string()
        })
        .await
        .unwrap()
        .is_none()
    );
}

#[tokio::test]
async fn machine_probe_failures_are_terminal_and_only_recent_jobs_are_retained() {
    let (mut context, _root) = TestAppBuilder::new("machine-probe-errors")
        .build_with_root()
        .await;
    context.app_config.testing.order = vec![ConnectionTestStage::Tcp, ConnectionTestStage::Tcp];
    let jobs = ProbeJobs::new(context);
    let mut first = None;
    for _ in 0..18 {
        let start = jobs
            .dispatch(ProbeRequest::Start {
                config_ids: vec![ConfigId(1)],
                stages: Default::default(),
            })
            .await
            .unwrap()
            .unwrap();
        first.get_or_insert(start.job_id.clone());
        let failed = terminal(&jobs, &start.job_id).await;
        assert_eq!(failed.state, ProbeState::Failed);
        assert!(failed.error.is_some());
        tokio::task::yield_now().await;
    }
    assert!(
        jobs.dispatch(ProbeRequest::Status {
            job_id: first.clone()
        })
        .await
        .unwrap()
        .is_none()
    );
    assert!(
        jobs.progress(first.as_deref().unwrap(), 0, 200)
            .await
            .unwrap()
            .is_none()
    );
    jobs.shutdown().await.unwrap();
}

#[test]
fn machine_probe_schema_accepts_no_policy_path_or_network_overrides() {
    for field in [
        "test_url",
        "download_url",
        "upload_url",
        "concurrency",
        "tcp_timeout_ms",
        "engine_path",
        "config_path",
    ] {
        let request = serde_json::json!({"type":"start", "config_ids":[1], "stages":{"skip_icmp":false,"skip_tcp":false,"skip_real_delay":false,"skip_download":false,"skip_upload":false}, (field):"untrusted"});
        assert!(serde_json::from_value::<ProbeRequest>(request).is_err());
    }
    assert!(
        serde_json::from_value::<ProbeStages>(
            serde_json::json!({"skip_icmp":false,"skip_tcp":true,"skip_real_delay":false,"skip_download":false,"skip_upload":false,"upload_url":"https://untrusted"})
        )
        .is_err()
    );
}
