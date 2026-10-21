use std::path::PathBuf;
use tokio::sync::mpsc;

use super::{GATEWAY_VERSION, GatewayPayload, GatewayRequest, GatewayRequestKind};
use crate::app::daemon::{ipc, supervisor::SupervisorEvent};
use crate::app::{AppError, Result};

pub struct MachineGateway {
    operator_sid: String,
    config_path: PathBuf,
    supervisor: mpsc::Sender<SupervisorEvent>,
    catalog: super::catalog::MachineCatalog,
    diagnostics: crate::app::services::diagnostics::DiagnosticsService,
}

impl MachineGateway {
    #[cfg(test)]
    pub(crate) fn for_test(
        context: crate::app::context::AppContext,
        operator_sid: &str,
        supervisor: mpsc::Sender<SupervisorEvent>,
    ) -> Self {
        Self {
            operator_sid: operator_sid.into(),
            config_path: context.runtime_paths.config_path.clone(),
            supervisor,
            diagnostics: crate::app::services::diagnostics::DiagnosticsService::new(
                context.clone(),
            ),
            catalog: super::catalog::MachineCatalog::new(context),
        }
    }
    #[cfg(windows)]
    pub fn new(
        profile: &super::MachineProfile,
        supervisor: mpsc::Sender<SupervisorEvent>,
        context: crate::app::context::AppContext,
    ) -> Result<Self> {
        if xrat_support::windows::current_user_sid()? != super::SYSTEM_SID {
            return Err(AppError::InvalidArgument(
                "machine gateway requires LocalSystem".into(),
            ));
        }
        Ok(Self {
            operator_sid: profile.operator_sid().to_owned(),
            config_path: profile.config_path(),
            supervisor,
            diagnostics: crate::app::services::diagnostics::DiagnosticsService::new(
                context.clone(),
            ),
            catalog: super::catalog::MachineCatalog::new(context),
        })
    }

    pub async fn dispatch(
        &self,
        peer_sid: &str,
        request: GatewayRequest,
    ) -> Result<ipc::DaemonResponse<GatewayPayload>> {
        if peer_sid != self.operator_sid {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "machine gateway caller is not the installing user",
            )
            .into());
        }
        if request.protocol_version != GATEWAY_VERSION {
            return Err(AppError::InvalidArgument(
                "unsupported machine gateway protocol version".into(),
            ));
        }
        let request = match request.request {
            GatewayRequestKind::Catalog { request } => {
                return Ok(completed(GatewayPayload::Catalog(
                    self.catalog.dispatch(request).await?,
                )));
            }
            GatewayRequestKind::Diagnostics { request } => {
                return Ok(completed(GatewayPayload::Diagnostics(
                    self.diagnostics.dispatch(request).await?,
                )));
            }
            request => request,
        };
        #[cfg(any(unix, windows))]
        {
            use ipc::transport::*;
            let supervisor = self.supervisor.clone();
            Ok(match request {
                GatewayRequestKind::Catalog { .. } | GatewayRequestKind::Diagnostics { .. } => {
                    return Err(AppError::InvalidArgument(
                        "catalog dispatch invariant failed".into(),
                    ));
                }
                GatewayRequestKind::Overview {} => {
                    let overview = ipc::transport::roundtrip(supervisor, |respond_to| {
                        SupervisorEvent::RuntimeOverview { respond_to }
                    })
                    .await?
                    .map_err(AppError::InvalidArgument)?;
                    completed(GatewayPayload::Overview(overview))
                }
                GatewayRequestKind::Ping {} => map(
                    ping_response_via_supervisor(supervisor).await?,
                    GatewayPayload::Ping,
                ),
                GatewayRequestKind::Status {} => map(
                    runtime_status_response_via_supervisor(supervisor).await?,
                    GatewayPayload::Status,
                ),
                GatewayRequestKind::Connect { config_id } => {
                    if config_id.0 <= 0 {
                        return Err(AppError::InvalidArgument(
                            "config ID must be positive".into(),
                        ));
                    }
                    map(
                        runtime_connect_response_via_supervisor(supervisor, config_id).await?,
                        GatewayPayload::Connect,
                    )
                }
                GatewayRequestKind::Disconnect {} => map(
                    runtime_disconnect_response_via_supervisor(supervisor).await?,
                    GatewayPayload::Disconnect,
                ),
                GatewayRequestKind::Replace { candidate_id } => {
                    if candidate_id.is_some_and(|id| id.0 <= 0) {
                        return Err(AppError::InvalidArgument(
                            "candidate ID must be positive".into(),
                        ));
                    }
                    map(
                        runtime_replace_response_via_supervisor(
                            supervisor,
                            ipc::RotationTrigger::Manual,
                            candidate_id,
                        )
                        .await?,
                        GatewayPayload::Replace,
                    )
                }
                GatewayRequestKind::Tun { enabled } => map(
                    runtime_tun_response_via_supervisor(
                        supervisor,
                        Some(enabled),
                        self.config_path.clone(),
                    )
                    .await?,
                    GatewayPayload::Tun,
                ),
                GatewayRequestKind::RotationStart {} => map(
                    proxy_start_response_via_supervisor(supervisor).await?,
                    GatewayPayload::RotationControl,
                ),
                GatewayRequestKind::RotationStatus {} => map(
                    proxy_status_response_via_supervisor(supervisor).await?,
                    GatewayPayload::RotationStatus,
                ),
                GatewayRequestKind::RotationStop {} => map(
                    proxy_stop_response_via_supervisor(supervisor).await?,
                    GatewayPayload::RotationControl,
                ),
            })
        }
        #[cfg(not(any(unix, windows)))]
        Err(AppError::InvalidArgument(
            "machine gateway requires Windows".into(),
        ))
    }
}

fn completed(payload: GatewayPayload) -> ipc::DaemonResponse<GatewayPayload> {
    ipc::DaemonResponse {
        protocol_version: GATEWAY_VERSION,
        ok: true,
        code: ipc::DaemonResponseCode::Ok,
        message: "machine operation completed".into(),
        payload: Some(payload),
    }
}

fn map<T>(
    response: ipc::DaemonResponse<T>,
    payload: impl FnOnce(T) -> GatewayPayload,
) -> ipc::DaemonResponse<GatewayPayload> {
    ipc::DaemonResponse {
        protocol_version: GATEWAY_VERSION,
        ok: response.ok,
        code: response.code,
        message: response.message,
        payload: response.payload.map(payload),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
