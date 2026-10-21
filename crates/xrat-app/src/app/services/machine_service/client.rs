use super::{
    CatalogPayload, CatalogRequest, GATEWAY_VERSION, GatewayPayload, GatewayRequest,
    GatewayRequestKind,
};
use crate::app::daemon::ipc::{DaemonResponse, DaemonResponseCode};
use crate::app::services::{ConfigListRequest, ConfigListResult};
use crate::app::{AppError, Result};
use std::sync::Arc;
use xrat_db::{ConfigRecord, SubscriptionRecord};
use xrat_model::{ConfigId, ConfigRef};

use super::catalog::PAGE_LIMIT;

#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to boxed futures"
)]
#[async_trait::async_trait]
pub(crate) trait Transport: Send + Sync {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>>;
}

#[cfg(windows)]
struct PipeTransport;

#[cfg(windows)]
#[async_trait::async_trait]
impl Transport for PipeTransport {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        super::request(request).await
    }
}

#[derive(Clone)]
pub struct MachineClient {
    transport: Arc<dyn Transport>,
}

#[cfg(windows)]
impl Default for MachineClient {
    fn default() -> Self {
        Self {
            transport: Arc::new(PipeTransport),
        }
    }
}

impl MachineClient {
    #[cfg(test)]
    pub(crate) fn with_transport(transport: Arc<dyn Transport>) -> Self {
        Self { transport }
    }
    #[cfg(windows)]
    pub fn new() -> Self {
        Self::default()
    }

    async fn exchange(&self, request: CatalogRequest) -> Result<DaemonResponse<GatewayPayload>> {
        self.exchange_kind(GatewayRequestKind::Catalog { request })
            .await
    }

    async fn exchange_kind(
        &self,
        request: GatewayRequestKind,
    ) -> Result<DaemonResponse<GatewayPayload>> {
        let response = self
            .transport
            .exchange(GatewayRequest {
                protocol_version: GATEWAY_VERSION,
                request,
            })
            .await
            .map_err(|error| {
                let mut text = error.to_string();
                let mut source = std::error::Error::source(&error);
                while let Some(cause) = source {
                    text.push_str(": ");
                    text.push_str(&cause.to_string());
                    source = cause.source();
                }
                AppError::InvalidArgument(format!(
                    "machine service request failed: {text}; the user profile was not opened"
                ))
            })?;
        if response.protocol_version != GATEWAY_VERSION {
            return Err(AppError::InvalidArgument(
                "machine service protocol mismatch; upgrade/restart the service before retrying"
                    .into(),
            ));
        }
        Ok(response)
    }

    fn payload(response: DaemonResponse<GatewayPayload>) -> Result<CatalogPayload> {
        if !response.ok {
            return Err(AppError::InvalidArgument(response.message));
        }
        match response.payload {
            Some(GatewayPayload::Catalog(payload)) => Ok(payload),
            _ => Err(AppError::InvalidArgument(
                "machine service returned an unexpected catalog payload".into(),
            )),
        }
    }

    pub async fn catalog(&self, request: CatalogRequest) -> Result<CatalogPayload> {
        Self::payload(self.exchange(request).await?)
    }

    pub async fn call(&self, request: GatewayRequestKind) -> Result<GatewayPayload> {
        let response = self.exchange_kind(request).await?;
        if !response.ok {
            return Err(AppError::InvalidArgument(response.message));
        }
        response
            .payload
            .ok_or_else(|| AppError::InvalidArgument("machine service returned no payload".into()))
    }

    pub async fn diagnostics(
        &self,
        request: crate::app::services::diagnostics::DiagnosticsRequest,
    ) -> Result<crate::app::services::diagnostics::DiagnosticsPayload> {
        match self
            .call(GatewayRequestKind::Diagnostics { request })
            .await?
        {
            GatewayPayload::Diagnostics(payload) => Ok(payload),
            _ => Err(unexpected()),
        }
    }

    pub async fn config_reference(&self, id: ConfigId) -> Result<Option<String>> {
        match self
            .catalog(CatalogRequest::Reference { config_id: id })
            .await?
        {
            CatalogPayload::Reference(reference) => Ok(reference),
            _ => Err(unexpected()),
        }
    }

    pub async fn resolve_config(&self, reference: &ConfigRef) -> Result<ConfigId> {
        match self
            .catalog(CatalogRequest::ResolveConfig {
                reference: reference.clone(),
            })
            .await?
        {
            CatalogPayload::ConfigId(id) => Ok(id),
            _ => Err(unexpected()),
        }
    }

    pub async fn config(&self, reference: &ConfigRef) -> Result<ConfigRecord> {
        match self
            .catalog(CatalogRequest::Config {
                reference: reference.clone(),
            })
            .await?
        {
            CatalogPayload::Config(record) => Ok(*record),
            _ => Err(unexpected()),
        }
    }

    pub async fn list(&self, filter: &ConfigListRequest) -> Result<ConfigListResult> {
        let (mut offset, goal) = selection(filter)?;
        let mut limit = PAGE_LIMIT;
        let mut items = Vec::new();
        let total = loop {
            let remaining = goal.map(|goal| goal.saturating_sub(items.len() as i64));
            let requested = remaining
                .map(|remaining| remaining.max(1).min(limit))
                .unwrap_or(limit);
            let mut page_filter = filter.clone();
            page_filter.offset = Some(offset);
            page_filter.limit = Some(requested);
            let response = self
                .exchange(CatalogRequest::List {
                    filter: page_filter,
                })
                .await?;
            if matches!(response.code, DaemonResponseCode::MessageTooLarge) && limit > 1 {
                limit = (limit / 2).max(1);
                continue;
            }
            let CatalogPayload::Configs(page) = Self::payload(response)? else {
                return Err(unexpected());
            };
            if page.total < 0 || page.items.len() > requested as usize {
                return Err(unexpected());
            }
            let received = page.items.len();
            if goal != Some(0) {
                items.extend(page.items);
            }
            offset = offset.checked_add(received as i64).ok_or_else(unexpected)?;
            if received == 0
                || offset >= page.total
                || goal.is_some_and(|goal| items.len() as i64 >= goal)
            {
                break page.total;
            }
        };
        Ok(ConfigListResult { total, items })
    }

    pub async fn export(&self, filter: &ConfigListRequest) -> Result<Vec<String>> {
        // Exports include the entire filtered view; listing pagination is presentation only.
        let mut offset = 0;
        let mut limit = PAGE_LIMIT;
        let mut items = Vec::new();
        loop {
            let mut page_filter = filter.clone();
            page_filter.offset = Some(offset);
            page_filter.limit = Some(limit);
            let response = self
                .exchange(CatalogRequest::ExportPage {
                    filter: page_filter,
                })
                .await?;
            if matches!(response.code, DaemonResponseCode::MessageTooLarge) && limit > 1 {
                limit = (limit / 2).max(1);
                continue;
            }
            let CatalogPayload::ExportPage { total, items: page } = Self::payload(response)? else {
                return Err(unexpected());
            };
            if total < 0 || page.len() > limit as usize {
                return Err(unexpected());
            }
            let received = page.len();
            items.extend(page);
            offset = offset.checked_add(received as i64).ok_or_else(unexpected)?;
            if received == 0 || offset >= total {
                return Ok(items);
            }
        }
    }

    pub async fn subscriptions(&self) -> Result<Vec<SubscriptionRecord>> {
        let mut offset = 0u64;
        let mut limit = PAGE_LIMIT as u16;
        let mut items = Vec::new();
        loop {
            let response = self
                .exchange(CatalogRequest::Subscriptions { offset, limit })
                .await?;
            if matches!(response.code, DaemonResponseCode::MessageTooLarge) && limit > 1 {
                limit = (limit / 2).max(1);
                continue;
            }
            let CatalogPayload::Subscriptions { total, items: page } = Self::payload(response)?
            else {
                return Err(unexpected());
            };
            if page.len() > limit as usize {
                return Err(unexpected());
            }
            let received = page.len();
            items.extend(page);
            offset = offset.checked_add(received as u64).ok_or_else(unexpected)?;
            if received == 0 || offset >= total {
                return Ok(items);
            }
        }
    }
}

fn selection(filter: &ConfigListRequest) -> Result<(i64, Option<i64>)> {
    if filter.top_by_real_delay.is_some() {
        return Ok((0, None));
    }
    match (filter.offset, filter.limit) {
        (Some(offset), Some(limit)) if offset >= 0 && limit >= 0 => Ok((offset, Some(limit))),
        (Some(_), Some(_)) => Err(AppError::InvalidArgument(
            "catalog pagination cannot be negative".into(),
        )),
        _ => Ok((0, None)),
    }
}

fn unexpected() -> AppError {
    AppError::InvalidArgument("machine service returned an invalid catalog page or payload".into())
}

#[cfg(test)]
mod tests;

mod imports;

mod catalog;
