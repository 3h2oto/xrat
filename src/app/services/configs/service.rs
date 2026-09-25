use std::sync::Arc;

use crate::app::ports::ConfigRepository;
use crate::app::read_models::ConfigSummary;
use crate::app::{AppError, Result};
use crate::db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};

use super::repository::DatabaseConfigRepository;

/// Filter and pagination request for listing configs.
///
/// Adapters translate their own input (CLI args, HTTP query params, TUI state)
/// into this struct so filter defaults and pagination rules live in one place.
#[derive(Clone, Debug, Default)]
pub struct ConfigListRequest {
    pub only_enabled: bool,
    pub only_active: bool,
    pub only_deleted: bool,
    pub include_deleted: bool,
    pub subscription_id: Option<i64>,
    pub protocol: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

impl ConfigListRequest {
    pub fn filter(&self) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.only_enabled,
            only_active: self.only_active,
            only_deleted: self.only_deleted,
            include_deleted: self.include_deleted,
            subscription_id: self.subscription_id,
            protocol: self.protocol.clone(),
        }
    }
}

/// Result of a config list query.
#[derive(Clone, Debug)]
pub struct ConfigListResult {
    pub total: i64,
    pub items: Vec<ConfigWithLatestTest>,
    pub summaries: Vec<ConfigSummary>,
}

/// Owns config read rules shared by CLI, HTTP, and TUI.
#[derive(Clone)]
pub struct ConfigService {
    repository: Arc<DatabaseConfigRepository>,
}

impl ConfigService {
    pub fn new(repository: DatabaseConfigRepository) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// List configs with the latest test joined, applying filter and pagination.
    pub async fn list(&self, request: &ConfigListRequest) -> Result<ConfigListResult> {
        let filter = request.filter();
        let total = self.repository.count_filtered_configs(&filter).await?;

        let items = match (request.offset, request.limit) {
            (Some(offset), Some(limit)) => {
                self.repository
                    .list_configs_paginated_with_latest_tests(&filter, offset, limit)
                    .await?
            }
            _ => {
                self.repository
                    .list_configs_with_latest_tests(&filter)
                    .await?
            }
        };

        let summaries = items.iter().map(ConfigSummary::from_joined).collect();
        Ok(ConfigListResult {
            total,
            items,
            summaries,
        })
    }

    /// Fetch a single config with its latest test.
    pub async fn detail(&self, id: i64) -> Result<Option<ConfigWithLatestTest>> {
        Ok(self.repository.get_config_with_latest_test(id).await?)
    }

    /// Resolve a numeric id or ref prefix into a config id.
    pub async fn resolve_id(&self, raw: &str) -> Result<Option<i64>> {
        if let Ok(id) = raw.parse::<i64>() {
            let exists = self.repository.get_config_by_id(id).await?.is_some();
            return Ok(exists.then_some(id));
        }

        if !crate::support::refs::is_ref_prefix(raw) {
            return Ok(None);
        }

        match self.repository.resolve_config_ref_prefix(raw).await? {
            RefMatch::Unique(id) => Ok(Some(id)),
            RefMatch::Ambiguous => Err(AppError::InvalidArgument(format!(
                "config ref prefix '{raw}' is ambiguous; provide more characters"
            ))),
            RefMatch::None => Ok(None),
        }
    }

    /// List subscriptions for ref enrichment.
    pub async fn subscriptions(&self) -> Result<Vec<crate::db::SubscriptionRecord>> {
        Ok(self.repository.list_subscriptions().await?)
    }
}
