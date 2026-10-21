use super::{GatewayPayload, GatewayRequestKind, MachineClient, Result, unexpected};
use crate::app::services::machine_service::{ProbeRequest, ProbeSnapshot, ProbeStages};
use xrat_model::ConfigId;

impl MachineClient {
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
