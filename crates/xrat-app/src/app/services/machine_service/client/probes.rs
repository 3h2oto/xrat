use super::{GatewayPayload, GatewayRequestKind, MachineClient, Result, unexpected};
use crate::app::services::machine_service::{ProbeRequest, ProbeSnapshot, ProbeStages};
use xrat_model::ConfigId;

impl MachineClient {
    pub async fn probe_progress(
        &self,
        job_id: &str,
    ) -> Result<Option<Vec<xrat_web::types::ConfigTestProgress>>> {
        let mut offset = 0;
        let mut rows = std::collections::BTreeMap::new();
        loop {
            let response = self
                .call(GatewayRequestKind::ProbeProgress {
                    job_id: job_id.into(),
                    offset,
                    limit: 200,
                })
                .await?;
            let GatewayPayload::ProbeProgress(page) = response else {
                return Err(unexpected());
            };
            let Some(page) = page else {
                return if offset == 0 {
                    Ok(None)
                } else {
                    Err(crate::app::AppError::InvalidArgument(
                        "probe job expired during progress read".into(),
                    ))
                };
            };
            if page.total > 10_000
                || page.rows.len() > 200
                || page
                    .rows
                    .iter()
                    .any(|row| row.config_id <= 0 || row.metrics.len() > 5)
            {
                return Err(unexpected());
            }
            let received = page.rows.len();
            for row in page.rows {
                rows.insert(row.config_id, row);
            }
            offset += received as u32;
            if received == 0 || offset >= page.total {
                return Ok(Some(rows.into_values().collect()));
            }
        }
    }

    pub(crate) async fn probe_results(
        &self,
        job_id: &str,
    ) -> Result<Option<Vec<crate::app::services::testing::TestOutputRow>>> {
        use crate::app::daemon::ipc::DaemonResponseCode;
        let mut offset = 0u32;
        let mut limit = 200u16;
        let mut expected = None;
        let mut rows = Vec::new();
        let mut ids = std::collections::HashSet::new();
        loop {
            let response = self
                .exchange_kind(GatewayRequestKind::ProbeResults {
                    job_id: job_id.into(),
                    offset,
                    limit,
                })
                .await?;
            if matches!(response.code, DaemonResponseCode::MessageTooLarge) && limit > 1 {
                limit = (limit / 2).max(1);
                continue;
            }
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let Some(GatewayPayload::ProbeResults(page)) = response.payload else {
                return Err(unexpected());
            };
            let Some(page) = page else {
                return if offset == 0 {
                    Ok(None)
                } else {
                    Err(crate::app::AppError::InvalidArgument(
                        "probe output was replaced during result read".into(),
                    ))
                };
            };
            if page.total > 10_000
                || expected.is_some_and(|expected| expected != page.total)
                || page.rows.len() > usize::from(limit)
                || offset + page.rows.len() as u32 > page.total
                || (page.rows.is_empty() && offset < page.total)
            {
                return Err(unexpected());
            }
            expected = Some(page.total);
            for row in page.rows {
                if row.output.id.0 <= 0 || !ids.insert(row.output.id) {
                    return Err(unexpected());
                }
                rows.push(row.output);
            }
            offset = rows.len() as u32;
            if offset == page.total {
                return Ok(Some(rows));
            }
        }
    }

    pub async fn probes(&self, request: ProbeRequest) -> Result<Option<ProbeSnapshot>> {
        match self.call(GatewayRequestKind::Probes { request }).await? {
            GatewayPayload::Probes(snapshot) => Ok(snapshot),
            _ => Err(unexpected()),
        }
    }

    pub async fn start_probe(
        &self,
        config_ids: Vec<ConfigId>,
        stages: ProbeStages,
    ) -> Result<ProbeSnapshot> {
        self.probes(ProbeRequest::Start { config_ids, stages })
            .await?
            .ok_or_else(unexpected)
    }

    pub async fn probe_status(&self, job_id: Option<String>) -> Result<Option<ProbeSnapshot>> {
        self.probes(ProbeRequest::Status { job_id }).await
    }

    pub async fn cancel_probe(&self, job_id: String) -> Result<Option<ProbeSnapshot>> {
        self.probes(ProbeRequest::Cancel { job_id }).await
    }
}
