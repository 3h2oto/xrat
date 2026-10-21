use super::run;
use crate::app::Result;
use crate::app::context::AppContext;
use crate::app::daemon::ipc::{DaemonResponse, DaemonResponseCode};
use crate::app::services::machine_service::{
    CatalogRequest, GATEWAY_VERSION, GatewayPayload, GatewayRequest, GatewayRequestKind,
    MachineClient, MachineGateway, Transport,
};
use crate::app::tests::TestAppBuilder;
use crate::cli::Cli;
use clap::Parser;
use std::sync::{Arc, Mutex};

const OPERATOR: &str = "S-1-5-21-1-2-3-1001";
const NODE: &str =
    "vless://00000000-0000-0000-0000-000000000001@one.example:443?security=tls#first";

struct AdapterTransport {
    gateway: MachineGateway,
    calls: Arc<Mutex<Vec<GatewayRequest>>>,
}

#[async_trait::async_trait]
impl Transport for AdapterTransport {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        self.calls.lock().unwrap().push(request.clone());
        let decoded = serde_json::from_slice(&serde_json::to_vec(&request)?)?;
        let response = self
            .gateway
            .dispatch(OPERATOR, decoded)
            .await
            .unwrap_or_else(|error| DaemonResponse {
                protocol_version: GATEWAY_VERSION,
                ok: false,
                code: DaemonResponseCode::InvalidState,
                message: error.to_string(),
                payload: None,
            });
        Ok(serde_json::from_slice(&serde_json::to_vec(&response)?)?)
    }
}

struct Fixture {
    context: AppContext,
    client: MachineClient,
    calls: Arc<Mutex<Vec<GatewayRequest>>>,
    supervisor: tokio::sync::mpsc::Receiver<crate::app::daemon::supervisor::SupervisorEvent>,
    _root: tempfile::TempDir,
}

async fn fixture() -> Fixture {
    let (context, root) = TestAppBuilder::new("machine-cli").build_with_root().await;
    let (sender, supervisor) = tokio::sync::mpsc::channel(4);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let client = MachineClient::with_transport(Arc::new(AdapterTransport {
        gateway: MachineGateway::for_test(context.clone(), OPERATOR, sender),
        calls: calls.clone(),
    }));
    Fixture {
        context,
        client,
        calls,
        supervisor,
        _root: root,
    }
}

async fn command(client: &MachineClient, arguments: &[&str]) -> Result<()> {
    let mut args = vec!["xrat", "--profile", "machine"];
    args.extend(arguments);
    let parsed = Cli::try_parse_from(args).unwrap();
    run(client, &parsed.command).await
}

#[tokio::test]
async fn machine_cli_catalog_adapters_use_shared_state_and_references() {
    let Fixture {
        context,
        client,
        calls,
        _root,
        ..
    } = fixture().await;
    command(&client, &["add", NODE]).await.unwrap();
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .remove(0);
    let reference = &config.r#ref[..8];
    command(&client, &["disable", reference]).await.unwrap();
    assert!(
        !context
            .db
            .get_config_by_id(config.id)
            .await
            .unwrap()
            .unwrap()
            .is_enabled
    );
    command(
        &client,
        &["list", "configs", "--enabled-only", "--format", "json"],
    )
    .await
    .unwrap();
    command(&client, &["show", "config", reference, "--json"])
        .await
        .unwrap();
    command(&client, &["enable", reference]).await.unwrap();
    command(&client, &["delete", "config", reference])
        .await
        .unwrap();
    assert!(
        context
            .db
            .get_config_by_id(config.id)
            .await
            .unwrap()
            .unwrap()
            .is_deleted
    );
    command(&client, &["restore", reference]).await.unwrap();
    assert!(
        !context
            .db
            .get_config_by_id(config.id)
            .await
            .unwrap()
            .unwrap()
            .is_deleted
    );
    command(
        &client,
        &[
            "list",
            "subscriptions",
            "--kind",
            "raw-text",
            "--format",
            "json",
        ],
    )
    .await
    .unwrap();
    assert!(calls.lock().unwrap().iter().any(|request|matches!(&request.request,GatewayRequestKind::Catalog {request:CatalogRequest::List {filter}} if filter.only_enabled)));
    command(&client, &["delete", "config", reference, "--hard"])
        .await
        .unwrap();
    assert!(
        context
            .db
            .get_config_by_id(config.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn machine_cli_acquires_file_content_in_the_client_and_preserves_source_metadata() {
    let Fixture {
        context,
        client,
        calls,
        _root: root,
        ..
    } = fixture().await;
    let path = root.path().join("operator input.txt");
    std::fs::write(&path, NODE).unwrap();
    command(
        &client,
        &["import", path.to_str().unwrap(), "--name", "uploaded"],
    )
    .await
    .unwrap();
    let subscription = context.db.list_subscriptions().await.unwrap().remove(0);
    assert_eq!(subscription.name.as_deref(), Some("uploaded"));
    assert_eq!(subscription.source_kind, "file");
    assert!(calls.lock().unwrap().iter().any(|request|matches!(&request.request,GatewayRequestKind::Catalog {request:CatalogRequest::Import {contents,..}} if contents==&[NODE.to_string()])));
    command(
        &client,
        &["show", "subscription", &subscription.r#ref, "--json"],
    )
    .await
    .unwrap();
    command(
        &client,
        &["delete", "subscription", &subscription.r#ref, "--yes"],
    )
    .await
    .unwrap();
    assert!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn machine_cli_rejects_unsafe_imports_without_user_profile_fallback() {
    let Fixture {
        context,
        client,
        calls,
        _root,
        ..
    } = fixture().await;
    let (user, _user_root) = TestAppBuilder::new("separate-user-profile")
        .build_with_root()
        .await;
    let before = calls.lock().unwrap().len();
    assert!(
        command(&client, &["add", r#"{"outbounds":[]}"#])
            .await
            .is_err()
    );
    assert_eq!(
        calls.lock().unwrap().len(),
        before,
        "raw JSON must fail before gateway dispatch"
    );
    let unsafe_node = "vless://00000000-0000-0000-0000-000000000001@one.example:443?type=xhttp&extra=%7B%22downloadSettings%22%3A%7B%7D%7D";
    assert!(command(&client, &["add", unsafe_node]).await.is_err());
    let error = command(&client, &["db", "migrate"]).await.unwrap_err();
    assert!(error.to_string().contains("user profile was not opened"));
    assert!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        user.db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn machine_cli_runtime_adapters_resolve_the_machine_catalog_and_use_the_supervisor() {
    use crate::app::daemon::{
        ipc,
        supervisor::{
            RuntimeConnectResult, RuntimeDisconnectResult, RuntimeStatusResult, SupervisorEvent,
        },
    };
    let Fixture {
        context,
        client,
        calls,
        mut supervisor,
        _root,
    } = fixture().await;
    command(&client, &["add", NODE]).await.unwrap();
    let record = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .remove(0);
    let id = record.id;
    let worker = tokio::spawn(async move {
        let SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        } = supervisor.recv().await.unwrap()
        else {
            panic!("expected connect");
        };
        assert_eq!(config_id, id);
        respond_to
            .send(RuntimeConnectResult::Ok(ipc::RuntimeConnectPayload {
                config_id,
                session_id: 7,
                pid: 42,
            }))
            .unwrap();
        let SupervisorEvent::RuntimeStatus { respond_to } = supervisor.recv().await.unwrap() else {
            panic!("expected status");
        };
        respond_to
            .send(RuntimeStatusResult::Ok(ipc::RuntimeStatusPayload {
                tun: None,
                daemon_ready: true,
                runtime_owned: true,
                runtime_status: "running".into(),
                session_id: Some(7),
                active_config_id: Some(id),
                pid_running: true,
                http_api_enabled: false,
                http_api_addr: None,
            }))
            .unwrap();
        let SupervisorEvent::RuntimeDisconnect { respond_to } = supervisor.recv().await.unwrap()
        else {
            panic!("expected disconnect");
        };
        respond_to
            .send(RuntimeDisconnectResult::Ok(ipc::RuntimeDisconnectPayload {
                stopped_session: true,
            }))
            .unwrap();
    });
    command(&client, &["connect", &record.r#ref[..8], "--json"])
        .await
        .unwrap();
    command(&client, &["status", "--json"]).await.unwrap();
    command(&client, &["disconnect", "--json"]).await.unwrap();
    worker.await.unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .filter(|request| matches!(request.request, GatewayRequestKind::Connect { .. }))
            .count(),
        1
    );
    assert!(calls.iter().any(|request|matches!(&request.request,GatewayRequestKind::Catalog {request:CatalogRequest::Reference {config_id}} if *config_id==id)));
}
