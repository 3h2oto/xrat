use super::*;
use crate::app::config::ConnectionTestStage;
use crate::app::services::machine_service::ProbeRequest;

#[tokio::test]
async fn machine_cli_probes_use_shared_filters_results_and_policy_without_ui_bootstrap() {
    let (mut context, root) = TestAppBuilder::new("machine-cli-probes")
        .build_with_root()
        .await;
    context.app_config.testing.order = vec![ConnectionTestStage::Tcp];
    context.app_config.testing.geoip.enabled = false;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let input = format!(
        "vless://00000000-0000-0000-0000-000000000001@127.0.0.1:{port}#one\nvless://00000000-0000-0000-0000-000000000002@127.0.0.1:{port}#two"
    );
    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::RawText,
        value: "CLI probe fixture".into(),
        name: None,
    };
    context
        .db
        .import_nodes(&source, &xrat_config::parse_text(&input))
        .await
        .unwrap();
    let records = context.db.list_configs(&Default::default()).await.unwrap();
    let (sender, _receiver) = tokio::sync::mpsc::channel(4);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let client = MachineClient::with_transport(Arc::new(AdapterTransport {
        gateway: MachineGateway::for_test(context.clone(), OPERATOR, sender),
        calls: calls.clone(),
    }));
    command(&client, &["disable", &records[1].id.to_string()])
        .await
        .unwrap();
    let output = root.path().join("probe-output.json");
    command(
        &client,
        &[
            "test",
            "--enabled-only",
            "--format",
            "json",
            "--output",
            output.to_str().unwrap(),
            "--no-progress",
        ],
    )
    .await
    .unwrap();
    let printed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    let status = client.probe_status(None).await.unwrap().unwrap();
    assert_eq!(status.completed, 1);
    let rows = client.probe_results(&status.job_id).await.unwrap().unwrap();
    assert_eq!(rows[0].id, records[0].id);
    assert!(rows[0].ran_tcp && rows[0].tcp_ok);
    assert_eq!(printed, serde_json::to_value(&rows).unwrap());
    assert_eq!(
        context
            .db
            .list_connection_tests_by_run(status.run_id.unwrap())
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(calls.lock().unwrap().iter().any(|call| matches!(&call.request, GatewayRequestKind::Catalog { request: CatalogRequest::List { filter } } if filter.only_enabled)));
    assert!(!root.path().join("machine-client").exists());
    command(
        &client,
        &[
            "test",
            "--latest-run-summary",
            "--country",
            "us",
            "--asn",
            "example",
        ],
    )
    .await
    .unwrap();
    assert_eq!(client.probe_status(None).await.unwrap().unwrap(), status);
    let invalid_output = root.path().join("invalid.json");
    assert!(
        command(
            &client,
            &[
                "test",
                "--concurrency",
                "9",
                "--output",
                invalid_output.to_str().unwrap()
            ]
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("administrator-controlled")
    );
    assert!(!invalid_output.exists());
    assert_eq!(client.probe_status(None).await.unwrap().unwrap(), status);
    let before = calls.lock().unwrap().len();
    assert!(
        command(&client, &["test", &records[0].r#ref, "--ping"])
            .await
            .unwrap_err()
            .to_string()
            .contains("--ping")
    );
    assert_eq!(calls.lock().unwrap().len(), before);
    command(&client, &["test", &records[0].r#ref, "--no-progress"])
        .await
        .unwrap();
    assert!(
        client
            .probe_results(&status.job_id)
            .await
            .unwrap()
            .is_none()
    );
    let next = client.probe_status(None).await.unwrap().unwrap();
    assert_ne!(next.run_id, status.run_id);
    assert_eq!(next.completed, 1);
    assert!(calls.lock().unwrap().iter().any(|call| matches!(&call.request, GatewayRequestKind::Probes { request: ProbeRequest::Start { config_ids, .. } } if config_ids == &[records[0].id])));
}

struct DeferredStatus {
    inner: AdapterTransport,
    first: std::sync::atomic::AtomicBool,
    entered: tokio::sync::Notify,
    cancellations: std::sync::atomic::AtomicUsize,
    reject_cancel: bool,
}
#[async_trait::async_trait]
impl Transport for DeferredStatus {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        use std::sync::atomic::Ordering;
        if matches!(
            &request.request,
            GatewayRequestKind::Probes {
                request: ProbeRequest::Status { .. }
            }
        ) && self.first.swap(false, Ordering::SeqCst)
        {
            self.entered.notify_one();
            std::future::pending::<()>().await;
        }
        if matches!(
            &request.request,
            GatewayRequestKind::Probes {
                request: ProbeRequest::Cancel { .. }
            }
        ) {
            self.cancellations.fetch_add(1, Ordering::SeqCst);
            if self.reject_cancel {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionReset,
                    "cancellation transport failure",
                )
                .into());
            }
        }
        self.inner.exchange(request).await
    }
}

#[tokio::test]
async fn machine_cli_probe_cancellation_interrupts_status_reads_and_requires_confirmation() {
    use std::sync::atomic::Ordering;
    for reject_cancel in [false, true] {
        let (mut context, root) = TestAppBuilder::new("machine-cli-cancel")
            .build_with_root()
            .await;
        context.app_config.testing.geoip.enabled = false;
        context.app_config.testing.order = vec![ConnectionTestStage::Tcp];
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let content = format!(
            "vless://00000000-0000-0000-0000-000000000001@127.0.0.1:{}#cancel",
            listener.local_addr().unwrap().port()
        );
        context
            .db
            .import_nodes(
                &xrat_db::ImportSource {
                    kind: xrat_db::SourceKind::RawText,
                    value: "cancel fixture".into(),
                    name: None,
                },
                &xrat_config::parse_text(&content),
            )
            .await
            .unwrap();
        let (sender, _receiver) = tokio::sync::mpsc::channel(4);
        let transport = Arc::new(DeferredStatus {
            inner: AdapterTransport {
                gateway: MachineGateway::for_test(context, OPERATOR, sender),
                calls: Arc::new(Mutex::new(Vec::new())),
            },
            first: std::sync::atomic::AtomicBool::new(true),
            entered: tokio::sync::Notify::new(),
            cancellations: std::sync::atomic::AtomicUsize::new(0),
            reject_cancel,
        });
        let client = MachineClient::with_transport(transport.clone());
        let output = root.path().join("cancel.json");
        let parsed = Cli::try_parse_from([
            "xrat",
            "test",
            "--no-progress",
            "--format",
            "json",
            "--output",
            output.to_str().unwrap(),
        ])
        .unwrap();
        let crate::cli::Command::Test(args) = parsed.command else {
            panic!("test args")
        };
        let signal = crate::app::tests::signals::FixtureShutdown::default();
        let operation = super::super::probes::run_with_signal(&client, &args, &signal);
        tokio::pin!(operation);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            tokio::select! {
                _ = signal.registered.notified() => {},
                result = &mut operation => panic!("CLI stopped before shutdown registration: {result:?}"),
            }
            tokio::select! {
                _ = transport.entered.notified() => {},
                result = &mut operation => panic!("CLI stopped before deferred status: {result:?}"),
            }
            signal.shutdown.notify_one();
            let result = operation.await;
            if reject_cancel {
                assert!(result.unwrap_err().to_string().contains("cancellation transport failure"));
                assert!(!output.exists());
            } else {
                result.unwrap();
                assert!(output.exists());
            }
        }).await.unwrap();
        assert_eq!(transport.cancellations.load(Ordering::SeqCst), 1);
        transport.inner.gateway.shutdown().await.unwrap();
    }
}
