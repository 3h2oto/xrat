use crate::app::daemon::ipc::{
    PingPayload, ProxyControlPayload, ProxyStatusPayload, RuntimeConnectPayload,
    RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload, TunStatePayload,
};
use serde::{Deserialize, Serialize};
use xrat_model::ConfigId;

pub const GATEWAY_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayRequest {
    pub protocol_version: u16,
    pub request: GatewayRequestKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum GatewayRequestKind {
    Catalog {
        request: super::CatalogRequest,
    },
    Diagnostics {
        request: crate::app::services::diagnostics::DiagnosticsRequest,
    },
    Probes {
        request: super::ProbeRequest,
    },
    ProbeProgress {
        job_id: String,
        offset: u32,
        limit: u16,
    },
    Overview {},
    Traffic {},
    Dashboard {},
    Ping {},
    Status {},
    Connect {
        config_id: ConfigId,
    },
    Disconnect {},
    Replace {
        candidate_id: Option<ConfigId>,
    },
    Tun {
        enabled: Option<bool>,
    },
    RotationStart {},
    RotationStatus {},
    RotationStop {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum GatewayPayload {
    Catalog(super::CatalogPayload),
    Diagnostics(crate::app::services::diagnostics::DiagnosticsPayload),
    Probes(Option<super::ProbeSnapshot>),
    ProbeProgress(Option<super::ProbeProgressPage>),
    Overview(Box<crate::app::read_models::RuntimeOverview>),
    Traffic(crate::app::read_models::TrafficSnapshot),
    Dashboard(xrat_web::types::DashboardData),
    Ping(PingPayload),
    Status(RuntimeStatusPayload),
    Connect(RuntimeConnectPayload),
    Disconnect(RuntimeDisconnectPayload),
    Replace(RuntimeReplacePayload),
    Tun(TunStatePayload),
    RotationControl(ProxyControlPayload),
    RotationStatus(ProxyStatusPayload),
}
