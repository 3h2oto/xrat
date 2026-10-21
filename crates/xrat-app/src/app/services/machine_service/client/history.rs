use super::{GatewayPayload, GatewayRequestKind, MachineClient, unexpected};
use crate::app::daemon::ipc::DaemonResponseCode;
use crate::app::services::diagnostics::{DiagnosticsPayload, DiagnosticsRequest};
use crate::app::{AppError, Result};
use xrat_db::{
    ConnectionTestCursor, ConnectionTestRecord, ConnectionTestRunRecord, ConnectionTestSelection,
};

impl MachineClient {
    pub async fn latest_probe_run(&self) -> Result<Option<ConnectionTestRunRecord>> {
        match self
            .diagnostics(DiagnosticsRequest::LatestProbeRun {})
            .await?
        {
            DiagnosticsPayload::LatestProbeRun(run) => Ok(run),
            _ => Err(unexpected()),
        }
    }

    pub async fn probe_history(
        &self,
        selection: ConnectionTestSelection,
        maximum: Option<usize>,
    ) -> Result<Vec<ConnectionTestRecord>> {
        let mut before: Option<ConnectionTestCursor> = None;
        let mut limit = 200u16;
        let mut rows = Vec::new();
        loop {
            let remaining = maximum.map(|maximum| maximum.saturating_sub(rows.len()));
            if remaining == Some(0) {
                return Ok(rows);
            }
            let requested =
                remaining.map_or(limit, |remaining| remaining.min(usize::from(limit)) as u16);
            let response = self
                .exchange_kind(GatewayRequestKind::Diagnostics {
                    request: DiagnosticsRequest::ProbePage {
                        selection,
                        before: before.clone(),
                        limit: requested,
                    },
                })
                .await?;
            if matches!(response.code, DaemonResponseCode::MessageTooLarge) && limit > 1 {
                limit = (limit / 2).max(1);
                continue;
            }
            if !response.ok {
                return Err(AppError::InvalidArgument(response.message));
            }
            let Some(GatewayPayload::Diagnostics(DiagnosticsPayload::ProbePage(page))) =
                response.payload
            else {
                return Err(unexpected());
            };
            if page.len() > usize::from(requested) {
                return Err(unexpected());
            }
            if let Some(last) = page.last() {
                let next = ConnectionTestCursor {
                    tested_at: last.tested_at.clone(),
                    id: last.id,
                };
                if before.as_ref().is_some_and(|before| {
                    (next.tested_at.as_str(), next.id) >= (before.tested_at.as_str(), before.id)
                }) {
                    return Err(unexpected());
                }
                before = Some(next);
            }
            let count = page.len();
            rows.extend(page);
            if count < usize::from(requested) {
                return Ok(rows);
            }
        }
    }
}
