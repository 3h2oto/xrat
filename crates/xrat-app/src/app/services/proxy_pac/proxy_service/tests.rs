use super::*;
use crate::app::daemon::ipc::DaemonResponse;
use crate::app::services::machine_service::{
    GatewayPayload, GatewayRequest, MachineClient, Transport,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Unavailable(AtomicUsize);
#[async_trait::async_trait]
impl Transport for Unavailable {
    async fn exchange(&self, _: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "owner offline").into())
    }
}

#[test]
fn proxy_preferences_are_read_only_and_invalid_files_do_not_fall_back() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("machine-client/config.toml");
    assert_eq!(operator_settings(&path).unwrap(), ServerSettings::default());
    assert!(!path.parent().unwrap().exists());
    std::fs::create_dir(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "[server]\nenabled=true\nport=19001\n[paths]\nxray='untrusted.exe'\n",
    )
    .unwrap();
    let settings = operator_settings(&path).unwrap();
    assert_eq!(settings.port, 19001);
    assert!(settings.enabled);
    std::fs::write(&path, "invalid = [").unwrap();
    assert!(operator_settings(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "invalid = [");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
}

#[tokio::test]
async fn machine_proxy_access_has_no_ready_owner_or_database_bootstrap() {
    let transport = Arc::new(Unavailable(AtomicUsize::new(0)));
    let service = ProxyService::for_machine_test(MachineClient::with_transport(transport.clone()));
    assert_eq!(transport.0.load(Ordering::SeqCst), 0);
    assert!(service.endpoints().await.is_err());
    assert!(service.pac().await.is_err());
    assert_eq!(transport.0.load(Ordering::SeqCst), 2);
}

struct Owner {
    gateway: crate::app::services::machine_service::MachineGateway,
    overview: std::sync::Mutex<crate::app::read_models::RuntimeOverview>,
}
#[async_trait::async_trait]
impl Transport for Owner {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        use crate::app::services::machine_service::{GATEWAY_VERSION, GatewayRequestKind};
        if matches!(request.request, GatewayRequestKind::Overview {}) {
            return Ok(DaemonResponse {
                protocol_version: GATEWAY_VERSION,
                ok: true,
                code: crate::app::daemon::ipc::DaemonResponseCode::Ok,
                message: "owner snapshot".into(),
                payload: Some(GatewayPayload::Overview(Box::new(
                    self.overview.lock().unwrap().clone(),
                ))),
            });
        }
        self.gateway.dispatch("S-1-5-21-1-2-3-1001", request).await
    }
}

async fn session(context: &AppContext, port: i64) -> xrat_db::RuntimeSessionRecord {
    context
        .db
        .insert_runtime_session(&xrat_db::RuntimeSessionInsert {
            config_id: None,
            status: xrat_db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".into()),
            socks_port: Some(port),
            http_host: Some("0.0.0.0".into()),
            http_port: Some(port + 1),
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: None,
            started_at: None,
            stopped_at: None,
        })
        .await
        .unwrap();
    context
        .db
        .get_running_runtime_session()
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn machine_proxy_reads_owner_endpoints_and_policy_while_retaining_frontend_api_preferences() {
    use crate::app::services::machine_service::MachineGateway;
    use crate::app::{
        runtime_service::{RuntimeService, RuntimeSessionDisplay},
        tests::TestAppBuilder,
    };
    let (mut owner, _owner_root) = TestAppBuilder::new("proxy-owner").build_with_root().await;
    owner.app_config.routing.block.domain = vec!["full:owner.blocked".into()];
    let mut runtime = RuntimeService::new(&owner).status().await.unwrap();
    let tun = crate::app::services::tun::capture_state(&owner, &runtime, &Default::default());
    runtime.session = Some(session(&owner, 21300).await);
    runtime.status = RuntimeSessionDisplay::Persisted(xrat_db::RuntimeSessionStatus::Running);
    runtime.pid_running = true;
    let (sender, _receiver) = tokio::sync::mpsc::channel(4);
    let transport = Arc::new(Owner {
        gateway: MachineGateway::for_test(owner.clone(), "S-1-5-21-1-2-3-1001", sender),
        overview: std::sync::Mutex::new(crate::app::read_models::RuntimeOverview {
            runtime,
            tun,
            daemon_ready: true,
            rotation_enabled: false,
            rotation_interval_secs: 60,
            test_stages: vec![],
            config_path: "protected owner".into(),
        }),
    });
    let (mut ui, _ui_root) = TestAppBuilder::new("proxy-ui").build_with_root().await;
    session(&ui, 31300).await;
    ui.app_config.routing.block.domain = vec!["full:ui.blocked".into()];
    ui.app_config.server.enabled = true;
    ui.app_config.server.port = 19444;
    ui.machine_client = Some(MachineClient::with_transport(transport.clone()));
    let service = ProxyService::from_context(&ui);
    assert_eq!(
        service.endpoints().await.unwrap(),
        active_endpoints(&owner.db).await.unwrap()
    );
    assert_ne!(
        service.endpoints().await.unwrap(),
        active_endpoints(&ui.db).await.unwrap()
    );
    let pac = service.pac().await.unwrap();
    assert!(pac.contains("owner.blocked"));
    assert!(!pac.contains("ui.blocked"));
    assert!(pac.contains("127.0.0.1:21300"));
    assert_eq!(service.server().unwrap().port, 19444);
    assert!(service.server().unwrap().enabled);
    transport.overview.lock().unwrap().runtime.pid_running = false;
    assert_eq!(
        service.endpoints().await.unwrap(),
        ActiveEndpoints::default()
    );
    assert!(ui.db.get_running_runtime_session().await.unwrap().is_some());
}
