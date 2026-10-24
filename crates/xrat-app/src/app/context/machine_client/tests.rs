use super::*;
use crate::app::daemon::ipc::DaemonResponse;
use crate::app::ports::ConfigRepository;
use crate::app::services::machine_service::{GatewayPayload, GatewayRequest, Transport};
use std::sync::Arc;

struct Loopback {
    gateway: crate::app::services::machine_service::MachineGateway,
    overview: crate::app::read_models::RuntimeOverview,
}

#[async_trait::async_trait]
impl Transport for Loopback {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        use crate::app::services::machine_service::{GATEWAY_VERSION, GatewayRequestKind};
        if matches!(request.request, GatewayRequestKind::Overview {}) {
            return Ok(DaemonResponse {
                protocol_version: GATEWAY_VERSION,
                ok: true,
                code: crate::app::daemon::ipc::DaemonResponseCode::Ok,
                message: "owner snapshot".into(),
                payload: Some(GatewayPayload::Overview(Box::new(self.overview.clone()))),
            });
        }
        self.gateway.dispatch("S-1-5-21-1-2-3-1001", request).await
    }
}

async fn loopback(context: &AppContext, ready: bool) -> MachineClient {
    let runtime = crate::app::runtime_service::RuntimeService::new(context)
        .status()
        .await
        .unwrap();
    let tun = crate::app::services::tun::capture_state(context, &runtime, &Default::default());
    let overview = crate::app::read_models::RuntimeOverview {
        runtime,
        tun,
        daemon_ready: ready,
        rotation_enabled: false,
        rotation_interval_secs: 60,
        test_stages: vec!["tcp".into()],
        config_path: context.runtime_paths.config_path.display().to_string(),
    };
    let (sender, _receiver) = tokio::sync::mpsc::channel(4);
    MachineClient::with_transport(Arc::new(Loopback {
        gateway: crate::app::services::machine_service::MachineGateway::for_test(
            context.clone(),
            "S-1-5-21-1-2-3-1001",
            sender,
        ),
        overview,
    }))
}

struct Unavailable;

#[async_trait::async_trait]
impl Transport for Unavailable {
    async fn exchange(&self, _: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        Err(
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "service unavailable")
                .into(),
        )
    }
}

#[tokio::test]
async fn machine_frontend_refuses_user_store_creation_when_service_is_unavailable() {
    let root = tempfile::tempdir().unwrap();
    let ui = root.path().join("machine-client");
    let client = MachineClient::with_transport(Arc::new(Unavailable));
    assert!(compose(client, &ui).await.is_err());
    assert!(!ui.exists());
}

#[tokio::test]
async fn machine_frontend_context_routes_services_and_never_uses_its_ui_store_as_catalog() {
    use crate::app::tests::{
        TestAppBuilder,
        fixtures::{test_node, test_source},
    };
    use crate::app::{
        runtime_service::RuntimeService,
        services::diagnostics::{DiagnosticsPayload, DiagnosticsRequest, DiagnosticsService},
    };
    let (owner, _owner_root) = TestAppBuilder::new("machine-frontend-owner")
        .build_with_root()
        .await;
    owner
        .db
        .import_nodes(&test_source(), &[test_node("owner.example")])
        .await
        .unwrap();
    owner
        .db
        .record_event(&xrat_db::NewEvent {
            level: "info".into(),
            source: "runtime".into(),
            kind: "owner".into(),
            config_id: None,
            session_id: None,
            message: "owner event".into(),
            detail: None,
        })
        .await
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let ui_root = root.path().join("machine-client");
    let not_ready = loopback(&owner, false).await;
    assert!(compose(not_ready, &ui_root).await.is_err());
    assert!(!ui_root.exists());
    let context = compose(loopback(&owner, true).await, &ui_root)
        .await
        .unwrap();
    assert_eq!(
        context.runtime_paths.database_path,
        ui_root.join("db.sqlite")
    );
    assert!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .is_empty()
    );
    let remote = context.services();
    let configs = remote.configs.list(&Default::default()).await.unwrap();
    assert_eq!(configs.total, 1);
    let summary = &configs.items[0].summary;
    assert_eq!(summary.address, "owner.example");
    remote
        .lifecycle
        .disable(&xrat_model::ConfigRef::from(summary.r#ref.as_str()))
        .await
        .unwrap();
    assert!(
        !owner
            .db
            .get_config_by_id(summary.id)
            .await
            .unwrap()
            .unwrap()
            .is_enabled
    );
    assert!(
        remote
            .configs
            .repository()
            .get_config_by_id(summary.id)
            .await
            .is_err()
    );
    let runtime = RuntimeService::new(&context);
    assert_eq!(
        runtime.status().await.unwrap().database_label,
        owner.runtime_paths.database_label
    );
    assert!(
        runtime
            .connect(crate::app::runtime_service::ConnectRequest {
                config_id: summary.id
            })
            .await
            .unwrap_err()
            .to_string()
            .contains("service gateway")
    );
    assert!(runtime.disconnect().await.is_err());
    for control in [
        crate::app::services::runtime_control::local_control(&context),
        crate::app::services::runtime_control::daemon_control(&context),
        crate::app::services::runtime_control::tui_control(&context)
            .await
            .unwrap(),
    ] {
        assert!(
            control.disconnect().await.is_err(),
            "unavailable owner must not become local disconnect"
        );
    }
    let diagnostics = DiagnosticsService::new(context.clone());
    let DiagnosticsPayload::Events(events) = diagnostics
        .dispatch(DiagnosticsRequest::Events {
            after: None,
            limit: 10,
            level: None,
        })
        .await
        .unwrap()
    else {
        panic!("events")
    };
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].message, "owner event");
    let dashboard = crate::app::services::dashboard::DashboardService::new(&context)
        .load(false)
        .await
        .unwrap();
    assert_eq!(dashboard.db_label, owner.runtime_paths.database_label);
    assert_eq!(
        dashboard.config_path,
        owner.runtime_paths.config_path.display().to_string()
    );
    assert!(dashboard.daemon.running);
    assert_eq!(dashboard.test_stages, vec!["tcp"]);
    assert_eq!(dashboard.configs.len(), 1);
    assert_eq!(dashboard.logs.events[0].message, "owner event");
    assert!(
        context
            .db
            .list_events(&xrat_db::EventFilter {
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        crate::app::services::runtime_traffic::collect(&context)
            .await
            .is_err()
    );
    assert!(
        crate::app::services::runtime_dashboard::collect(&context)
            .await
            .is_err()
    );
}

#[test]
fn machine_frontend_requires_an_ordinary_user_identity() {
    const OPERATOR: &str = "S-1-5-21-1-2-3-1001";
    assert!(validate_identity(OPERATOR, false).is_ok());
    assert!(validate_identity(OPERATOR, true).is_err());
    for sid in ["S-1-5-18", "S-1-5-19", "S-1-5-20", "S-1-5-32-544", ""] {
        assert!(validate_identity(sid, false).is_err());
    }
}

#[tokio::test]
async fn machine_frontend_probes_preserve_owner_stage_progress_and_reject_policy_overrides() {
    use crate::app::{config::ConnectionTestStage, services::testing, tests::TestAppBuilder};
    use xrat_support::cancel::{CancellationFlag, CancellationReceiver};
    let (mut owner, _owner_root) = TestAppBuilder::new("machine-frontend-probes")
        .build_with_root()
        .await;
    owner.app_config.testing.order = vec![ConnectionTestStage::Tcp];
    owner.app_config.testing.geoip.enabled = false;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (source, node) = crate::app::import::load_single_node(&format!(
        "vless://11111111-1111-1111-1111-111111111111@127.0.0.1:{}#probe",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    owner.db.import_nodes(&source, &[node]).await.unwrap();
    let id = owner.db.list_configs(&Default::default()).await.unwrap()[0].id;
    let root = tempfile::tempdir().unwrap();
    let context = compose(
        loopback(&owner, true).await,
        &root.path().join("machine-client"),
    )
    .await
    .unwrap();
    let client = context.machine_client.as_ref().unwrap();
    let cancelled = CancellationFlag::new();
    cancelled.cancel();
    let (updates, _) = tokio::sync::mpsc::unbounded_channel();
    assert_eq!(
        testing::run_bulk_for_config_ids_with_progress(
            &Default::default(),
            &context,
            &[id],
            CancellationReceiver::from_flag(&cancelled),
            updates
        )
        .await
        .unwrap(),
        0
    );
    assert!(client.probe_status(None).await.unwrap().is_none());
    let flag = CancellationFlag::new();
    let (updates, mut received) = tokio::sync::mpsc::unbounded_channel();
    let (activity, mut activities) = tokio::sync::mpsc::unbounded_channel();
    assert_eq!(
        testing::run_bulk_for_config_ids_with_activity(
            &Default::default(),
            &context,
            &[id],
            CancellationReceiver::from_flag(&flag),
            updates,
            activity
        )
        .await
        .unwrap(),
        1
    );
    let update = received.recv().await.unwrap();
    assert_eq!((update.config_id, update.done, update.total), (id, 1, 1));
    let mut progress = testing::TestActivityProgress::default();
    while let Some(update) = activities.recv().await {
        progress.apply(update);
    }
    assert!(
        progress.rows[&id.0]
            .metrics
            .iter()
            .any(|metric| metric.metric == xrat_web::types::TestMetric::Tcp
                && metric.state == xrat_web::types::TestMetricState::Passed)
    );
    assert!(
        context
            .db
            .get_latest_connection_test_run()
            .await
            .unwrap()
            .is_none()
    );
    let previous = client.probe_status(None).await.unwrap().unwrap();
    let (updates, _) = tokio::sync::mpsc::unbounded_channel();
    assert!(
        testing::run_bulk_for_config_ids_with_progress(
            &testing::TestRunRequest {
                test_url: Some("http://untrusted.example/".into()),
                ..Default::default()
            },
            &context,
            &[id],
            CancellationReceiver::from_flag(&flag),
            updates
        )
        .await
        .is_err()
    );
    assert_eq!(client.probe_status(None).await.unwrap().unwrap(), previous);
}

#[tokio::test]
async fn machine_frontend_web_actions_share_catalog_and_enforce_admin_policy() {
    use crate::app::{services::web_management, tests::TestAppBuilder};
    use xrat_support::cancel::CancellationFlag;
    use xrat_web::types::{Action, Engine};
    let (owner, _owner_root) = TestAppBuilder::new("machine-web-actions")
        .build_with_root()
        .await;
    let root = tempfile::tempdir().unwrap();
    let context = compose(loopback(&owner, true).await, &root.path().join("ui"))
        .await
        .unwrap();
    let flag = CancellationFlag::new();
    web_management::execute(
        &context,
        Action::Add {
            input: "vless://11111111-1111-1111-1111-111111111111@owner.example:443#remote".into(),
        },
        &flag,
        "test-job",
    )
    .await
    .unwrap();
    let rows = context
        .services()
        .configs
        .list(&Default::default())
        .await
        .unwrap();
    assert_eq!(rows.total, 1);
    let id = rows.items[0].summary.id.0;
    web_management::execute(&context, Action::Disable { id }, &flag, "test-job")
        .await
        .unwrap();
    assert!(
        !owner
            .db
            .get_config_by_id(id.into())
            .await
            .unwrap()
            .unwrap()
            .is_enabled
    );
    assert_eq!(context.db.get_config_count().await.unwrap(), 0);
    for action in [
        Action::InstallEngine {
            engine: Engine::Xray,
        },
        Action::Upgrade { version: None },
        Action::GeoIpUpdate,
        Action::Validate,
    ] {
        assert!(
            web_management::execute(&context, action, &flag, "test-job")
                .await
                .unwrap_err()
                .to_string()
                .contains("administrator")
        );
    }
    assert!(context.allows_setting("server.port"));
    assert!(context.allows_setting("tui.theme"));
    for path in [
        "runtime.tun.enabled",
        "testing.concurrency",
        "paths.xray",
        "routing.mode",
    ] {
        assert!(!context.allows_setting(path));
    }
    let data = crate::app::services::dashboard::DashboardService::new(&context)
        .load(false)
        .await
        .unwrap();
    assert!(data.test_concurrency_managed);
}

mod http;
