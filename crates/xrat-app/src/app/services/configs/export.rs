use crate::app::Result;
use crate::app::read_models::ConfigSummary;

use super::{ConfigListRequest, ConfigService, validate_top};

#[derive(Clone, Debug, Default)]
pub struct ConfigExportRequest {
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
    pub top: Option<u32>,
}

impl ConfigExportRequest {
    fn list_request(&self) -> Result<ConfigListRequest> {
        Ok(ConfigListRequest {
            only_enabled: self.enabled.unwrap_or(true),
            protocol: self.protocol.clone(),
            top_by_real_delay: self.top.map(validate_top).transpose()?,
            ..ConfigListRequest::default()
        })
    }
}

impl ConfigService {
    pub async fn export_summaries(
        &self,
        request: &ConfigExportRequest,
    ) -> Result<Vec<ConfigSummary>> {
        Ok(self
            .list(&request.list_request()?)
            .await?
            .items
            .into_iter()
            .map(|detail| detail.summary)
            .collect())
    }

    pub async fn export_subscription(&self, request: &ConfigExportRequest) -> Result<String> {
        Ok(self
            .export_raw_configs(&request.list_request()?)
            .await?
            .join("\n"))
    }
}

#[cfg(test)]
mod tests;
