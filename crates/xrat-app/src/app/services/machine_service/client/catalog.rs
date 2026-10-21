use super::{CatalogPayload, CatalogRequest, MachineClient, unexpected};
use crate::app::Result;
use crate::app::read_models::ConfigDetail;
use crate::app::services::{DeleteOutcome, RestoreOutcome, ToggleOutcome};
use xrat_db::{ConfigRecord, SubscriptionRecord};
use xrat_model::{ConfigId, ConfigRef, SubscriptionId};

impl MachineClient {
    pub(crate) async fn raw_configs_by_ids(&self, ids: &[ConfigId]) -> Result<Option<Vec<String>>> {
        let mut offset = 0;
        let mut limit = super::PAGE_LIMIT as usize;
        let mut items = Vec::new();
        while offset < ids.len() {
            let end = (offset + limit).min(ids.len());
            let response = self
                .exchange(CatalogRequest::RawConfigs {
                    ids: ids[offset..end].to_vec(),
                })
                .await?;
            if matches!(
                response.code,
                crate::app::daemon::ipc::DaemonResponseCode::MessageTooLarge
            ) && limit > 1
            {
                limit = (limit / 2).max(1);
                continue;
            }
            let CatalogPayload::RawConfigs(page) = Self::payload(response)? else {
                return Err(unexpected());
            };
            let Some(page) = page else {
                return Ok(None);
            };
            if page.len() != end - offset {
                return Err(unexpected());
            }
            items.extend(page);
            offset = end;
        }
        Ok(Some(items))
    }

    pub(crate) async fn config_detail(&self, id: ConfigId) -> Result<Option<ConfigDetail>> {
        match self
            .catalog(CatalogRequest::Detail { config_id: id })
            .await?
        {
            CatalogPayload::Detail(detail) => Ok(detail.map(|detail| *detail)),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn find_config(&self, reference: &ConfigRef) -> Result<Option<ConfigId>> {
        match self
            .catalog(CatalogRequest::FindConfig {
                reference: reference.clone(),
            })
            .await?
        {
            CatalogPayload::FoundConfig(id) => Ok(id),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn find_subscription(
        &self,
        reference: &str,
    ) -> Result<Option<SubscriptionId>> {
        match self
            .catalog(CatalogRequest::FindSubscription {
                reference: reference.into(),
            })
            .await?
        {
            CatalogPayload::FoundSubscription(id) => Ok(id),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn resolve_subscription(&self, reference: &str) -> Result<SubscriptionId> {
        match self
            .catalog(CatalogRequest::ResolveSubscription {
                reference: reference.into(),
            })
            .await?
        {
            CatalogPayload::SubscriptionId(id) => Ok(id),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn toggle_config(
        &self,
        reference: &ConfigRef,
        enabled: bool,
    ) -> Result<ToggleOutcome> {
        let reference = reference.clone();
        let request = if enabled {
            CatalogRequest::Enable { reference }
        } else {
            CatalogRequest::Disable { reference }
        };
        match self.catalog(request).await? {
            CatalogPayload::Toggle { outcome, .. } => Ok(outcome),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn delete_config(
        &self,
        reference: &ConfigRef,
        hard: bool,
    ) -> Result<(ConfigRecord, DeleteOutcome)> {
        match self
            .catalog(CatalogRequest::Delete {
                reference: reference.clone(),
                hard,
            })
            .await?
        {
            CatalogPayload::Delete { config, outcome } => Ok((*config, outcome)),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn restore_config(
        &self,
        reference: &ConfigRef,
    ) -> Result<(ConfigRecord, RestoreOutcome)> {
        match self
            .catalog(CatalogRequest::Restore {
                reference: reference.clone(),
            })
            .await?
        {
            CatalogPayload::Restore { config, outcome } => Ok((*config, outcome)),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn delete_subscription(&self, reference: &str) -> Result<SubscriptionRecord> {
        match self
            .catalog(CatalogRequest::DeleteSubscription {
                reference: reference.into(),
            })
            .await?
        {
            CatalogPayload::Subscription(record) => Ok(*record),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn rename_subscription(
        &self,
        reference: &str,
        name: &str,
    ) -> Result<SubscriptionRecord> {
        match self
            .catalog(CatalogRequest::RenameSubscription {
                reference: reference.into(),
                name: name.into(),
            })
            .await?
        {
            CatalogPayload::Subscription(record) => Ok(*record),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn delete_configs(&self, ids: &[ConfigId], hard: bool) -> Result<u64> {
        self.changed(CatalogRequest::DeleteMany {
            ids: ids.to_vec(),
            hard,
        })
        .await
    }

    pub(crate) async fn restore_configs(&self, ids: &[ConfigId]) -> Result<u64> {
        self.changed(CatalogRequest::RestoreMany { ids: ids.to_vec() })
            .await
    }

    pub(crate) async fn purge(&self) -> Result<u64> {
        self.changed(CatalogRequest::Purge {}).await
    }

    async fn changed(&self, request: CatalogRequest) -> Result<u64> {
        match self.catalog(request).await? {
            CatalogPayload::Changed(count) => Ok(count),
            _ => Err(unexpected()),
        }
    }
}
