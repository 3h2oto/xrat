use super::*;
use crate::app::read_models::TrafficSnapshot;
use xrat_web::types::{DashboardData, LatencyMetric, Traffic};

fn dashboard() -> DashboardData {
    DashboardData {
        sampled_at_ms: 123.0,
        session_id: Some(42),
        runtime_status: "running".into(),
        stats_enabled: true,
        tun_enabled: false,
        tun_active: false,
        tun_available: false,
        tun_reason: Some("driver not provisioned".into()),
        enabled_metrics: vec![LatencyMetric::Tcp],
        last_subscription_refresh: Some(23),
        probes: Vec::new(),
        traffic_history: Vec::new(),
    }
}

#[tokio::test]
async fn runtime_telemetry_gateway_requires_operator_and_preserves_owner_results() {
    let (gateway, mut receiver) = fixture().await;
    for kind in [
        GatewayRequestKind::Traffic {},
        GatewayRequestKind::Dashboard {},
    ] {
        assert!(
            gateway
                .dispatch("S-1-5-21-1-2-3-1002", request(kind))
                .await
                .is_err()
        );
    }
    assert!(receiver.try_recv().is_err());
    let traffic = TrafficSnapshot {
        session_id: Some(42),
        traffic: Some(Traffic {
            uploaded_bytes: 17,
            downloaded_bytes: 31,
        }),
    };
    let expected = traffic.clone();
    let worker = tokio::spawn(async move {
        let Some(SupervisorEvent::RuntimeTraffic { respond_to }) = receiver.recv().await else {
            panic!("traffic")
        };
        respond_to.send(Ok(traffic)).unwrap();
        let Some(SupervisorEvent::RuntimeDashboard { respond_to }) = receiver.recv().await else {
            panic!("dashboard")
        };
        respond_to.send(Ok(dashboard())).unwrap();
        let Some(SupervisorEvent::RuntimeTraffic { respond_to }) = receiver.recv().await else {
            panic!("traffic error")
        };
        respond_to.send(Err("owner unavailable".into())).unwrap();
        let Some(SupervisorEvent::RuntimeDashboard { respond_to }) = receiver.recv().await else {
            panic!("dashboard error")
        };
        respond_to.send(Err("owner unavailable".into())).unwrap();
    });
    let response = gateway
        .dispatch(OPERATOR, request(GatewayRequestKind::Traffic {}))
        .await
        .unwrap();
    let decoded: ipc::DaemonResponse<GatewayPayload> =
        serde_json::from_slice(&serde_json::to_vec(&response).unwrap()).unwrap();
    let Some(GatewayPayload::Traffic(actual)) = decoded.payload else {
        panic!("traffic payload")
    };
    assert_eq!(actual, expected);
    let response = gateway
        .dispatch(OPERATOR, request(GatewayRequestKind::Dashboard {}))
        .await
        .unwrap();
    let decoded: ipc::DaemonResponse<GatewayPayload> =
        serde_json::from_slice(&serde_json::to_vec(&response).unwrap()).unwrap();
    let Some(GatewayPayload::Dashboard(actual)) = decoded.payload else {
        panic!("dashboard payload")
    };
    assert_eq!(actual, dashboard());
    for kind in [
        GatewayRequestKind::Traffic {},
        GatewayRequestKind::Dashboard {},
    ] {
        assert!(
            gateway
                .dispatch(OPERATOR, request(kind))
                .await
                .unwrap_err()
                .to_string()
                .contains("owner unavailable")
        );
    }
    worker.await.unwrap();
    assert!(
        gateway
            .dispatch(OPERATOR, request(GatewayRequestKind::Traffic {}))
            .await
            .is_err()
    );
    for value in [
        serde_json::json!({"type":"traffic", "host":"untrusted", "port":80}),
        serde_json::json!({"type":"dashboard", "config_path":"untrusted"}),
    ] {
        assert!(serde_json::from_value::<GatewayRequestKind>(value).is_err());
    }
    gateway.shutdown().await.unwrap();
}
