use super::super::validate_operator;
use super::*;
use crate::app::daemon::supervisor::{RuntimeConnectResult, RuntimeDisconnectResult};
use xrat_model::ConfigId;

const OPERATOR: &str = "S-1-5-21-1-2-3-1001";

fn fixture() -> (MachineGateway, mpsc::Receiver<SupervisorEvent>) {
    let (supervisor, receiver) = mpsc::channel(4);
    (
        MachineGateway {
            operator_sid: OPERATOR.into(),
            config_path: "protected/config.toml".into(),
            supervisor,
        },
        receiver,
    )
}

fn request(request: GatewayRequestKind) -> GatewayRequest {
    GatewayRequest {
        protocol_version: GATEWAY_VERSION,
        request,
    }
}

#[test]
fn operator_sid_rejects_service_accounts_and_invalid_input() {
    assert!(validate_operator(OPERATOR).is_ok());
    for sid in [
        "",
        "S-1-",
        "S-1-5-18",
        "S-1-5-19",
        "S-1-5-20",
        "S-1-5-32-544",
        "S-1-5-21)(A;;GA;;;WD)",
        "S-1-5-4294967296",
    ] {
        assert!(validate_operator(sid).is_err(), "accepted {sid}");
    }
}

#[test]
fn wire_schema_rejects_paths_commands_sql_json_and_extra_fields() {
    for input in [
        r#"{"type":"tun","enabled":true,"config_path":"C:\\untrusted.toml"}"#,
        r#"{"type":"connect","config_id":1,"engine_json":{}}"#,
        r#"{"type":"connect","config_id":1,"executable":"cmd.exe"}"#,
        r#"{"type":"status","sql":"DROP TABLE configs"}"#,
        r#"{"type":"daemon_shutdown"}"#,
        r#"{"type":"replace","trigger":"health_check_failed"}"#,
    ] {
        assert!(
            serde_json::from_str::<GatewayRequestKind>(input).is_err(),
            "accepted {input}"
        );
    }
    assert!(
        serde_json::from_str::<GatewayRequest>(
            r#"{"protocol_version":1,"request":{"type":"status"},"path":"arbitrary"}"#
        )
        .is_err()
    );
}

#[tokio::test]
async fn rejects_other_users_versions_and_ids_before_supervisor_dispatch() {
    let (gateway, mut receiver) = fixture();
    for peer in ["S-1-5-21-1-2-3-1002", "S-1-5-18", "S-1-5-32-544", ""] {
        assert!(
            gateway
                .dispatch(peer, request(GatewayRequestKind::Status {}))
                .await
                .is_err()
        );
    }
    let mut wrong_version = request(GatewayRequestKind::Status {});
    wrong_version.protocol_version += 1;
    assert!(gateway.dispatch(OPERATOR, wrong_version).await.is_err());
    for id in [0, -1] {
        assert!(
            gateway
                .dispatch(
                    OPERATOR,
                    request(GatewayRequestKind::Connect {
                        config_id: ConfigId(id)
                    })
                )
                .await
                .is_err()
        );
        assert!(
            gateway
                .dispatch(
                    OPERATOR,
                    request(GatewayRequestKind::Replace {
                        candidate_id: Some(ConfigId(id))
                    })
                )
                .await
                .is_err()
        );
    }
    assert!(receiver.try_recv().is_err());
}

#[tokio::test]
async fn connect_and_repeated_disconnect_preserve_supervisor_results() {
    let (gateway, mut receiver) = fixture();
    let worker = tokio::spawn(async move {
        let Some(SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        }) = receiver.recv().await
        else {
            panic!("expected connect")
        };
        assert_eq!(config_id, ConfigId(42));
        respond_to
            .send(RuntimeConnectResult::Err {
                message: "config is disabled".into(),
            })
            .unwrap();
        for stopped_session in [true, false] {
            let Some(SupervisorEvent::RuntimeDisconnect { respond_to }) = receiver.recv().await
            else {
                panic!("expected disconnect")
            };
            respond_to
                .send(RuntimeDisconnectResult::Ok(ipc::RuntimeDisconnectPayload {
                    stopped_session,
                }))
                .unwrap();
        }
    });
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Connect {
                config_id: ConfigId(42),
            }),
        )
        .await
        .unwrap();
    assert!(!response.ok);
    assert_eq!(response.message, "config is disabled");
    for expected in [true, false] {
        let response = gateway
            .dispatch(OPERATOR, request(GatewayRequestKind::Disconnect {}))
            .await
            .unwrap();
        let Some(GatewayPayload::Disconnect(payload)) = response.payload else {
            panic!("missing disconnect result")
        };
        assert_eq!(payload.stopped_session, expected);
    }
    worker.await.unwrap();
}

#[tokio::test]
async fn tun_uses_only_service_owned_path_and_replace_is_manual() {
    let (gateway, mut receiver) = fixture();
    let worker = tokio::spawn(async move {
        let Some(SupervisorEvent::RuntimeTun {
            enabled,
            config_path,
            respond_to,
        }) = receiver.recv().await
        else {
            panic!("expected tun")
        };
        assert_eq!(enabled, Some(true));
        assert_eq!(config_path, PathBuf::from("protected/config.toml"));
        respond_to
            .send(Err("Windows TUN is not supported yet".into()))
            .unwrap();
        let Some(SupervisorEvent::RuntimeReplace {
            trigger,
            candidate_id,
            respond_to,
        }) = receiver.recv().await
        else {
            panic!("expected replace")
        };
        assert_eq!(trigger, ipc::RotationTrigger::Manual);
        assert_eq!(candidate_id, None);
        respond_to
            .send(crate::app::daemon::supervisor::RuntimeReplaceResult::Err {
                message: "no candidates".into(),
            })
            .unwrap();
    });
    let response = gateway
        .dispatch(OPERATOR, request(GatewayRequestKind::Tun { enabled: true }))
        .await
        .unwrap();
    assert!(!response.ok);
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Replace { candidate_id: None }),
        )
        .await
        .unwrap();
    assert!(!response.ok);
    worker.await.unwrap();
}

#[tokio::test]
async fn closed_supervisor_does_not_fall_back_to_local_execution() {
    let (gateway, receiver) = fixture();
    drop(receiver);
    assert!(
        gateway
            .dispatch(OPERATOR, request(GatewayRequestKind::Disconnect {}))
            .await
            .is_err()
    );
}
