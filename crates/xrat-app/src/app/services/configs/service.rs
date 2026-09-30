use std::sync::Arc;

use crate::app::ports::ConfigRepository;
use crate::app::read_models::ConfigDetail;
use crate::app::{AppError, Result};
use xrat_db::record::{ConfigListFilter, RefMatch};

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
    pub top_by_real_delay: Option<u32>,
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

/// Inclusive upper bound accepted for `top` selections.
pub const MAX_TOP: u32 = 200;

/// Validate a requested `top` count against the shared limit.
pub fn validate_top(top: u32) -> Result<u32> {
    if top == 0 || top > MAX_TOP {
        return Err(AppError::InvalidArgument(format!(
            "top must be between 1 and {MAX_TOP}"
        )));
    }
    Ok(top)
}

/// Result of a config list query.
#[derive(Clone, Debug)]
pub struct ConfigListResult {
    pub total: i64,
    pub items: Vec<ConfigDetail>,
}

/// Owns config read rules shared by CLI, HTTP, and TUI.
#[derive(Clone)]
pub struct ConfigService {
    repository: Arc<DatabaseConfigRepository>,
}

impl ConfigService {
    pub fn new(repository: Arc<DatabaseConfigRepository>) -> Self {
        Self { repository }
    }

    /// Shared repository access for sibling services.
    pub fn repository(&self) -> Arc<DatabaseConfigRepository> {
        Arc::clone(&self.repository)
    }

    /// List configs with the latest test joined, applying filter and pagination.
    pub async fn list(&self, request: &ConfigListRequest) -> Result<ConfigListResult> {
        let filter = request.filter();
        let items = if let Some(top) = request.top_by_real_delay {
            self.repository
                .list_top_configs_by_real_delay(i64::from(top), &filter)
                .await?
        } else if let (Some(offset), Some(limit)) = (request.offset, request.limit) {
            self.repository
                .list_configs_paginated_with_latest_tests(&filter, offset, limit)
                .await?
        } else {
            self.repository
                .list_configs_with_latest_tests(&filter)
                .await?
        };

        let total = if request.top_by_real_delay.is_some()
            || (request.offset.is_none() && request.limit.is_none())
        {
            items.len() as i64
        } else {
            self.repository.count_filtered_configs(&filter).await?
        };

        let items = items.iter().map(ConfigDetail::from_joined).collect();
        Ok(ConfigListResult { total, items })
    }

    /// Raw config links for a filtered selection, in listing order.
    pub async fn export_raw_configs(&self, request: &ConfigListRequest) -> Result<Vec<String>> {
        let filter = request.filter();
        if let Some(top) = request.top_by_real_delay {
            let rows = self
                .repository
                .list_top_configs_by_real_delay(i64::from(top), &filter)
                .await?;
            return Ok(rows.into_iter().map(|row| row.config.raw_config).collect());
        }

        let configs = self.repository.list_configs(&filter).await?;
        Ok(configs
            .into_iter()
            .map(|config| config.raw_config)
            .collect())
    }

    /// Fetch a config as an interface-neutral detail model.
    pub async fn detail(&self, id: i64) -> Result<Option<ConfigDetail>> {
        Ok(self
            .repository
            .get_config_with_latest_test(id)
            .await?
            .as_ref()
            .map(ConfigDetail::from_joined))
    }

    /// Resolve a numeric id or ref prefix into a config id.
    pub async fn resolve_id(&self, raw: &str) -> Result<Option<i64>> {
        if let Ok(id) = raw.parse::<i64>() {
            let exists = self.repository.get_config_by_id(id).await?.is_some();
            if exists {
                return Ok(Some(id));
            }
        }

        if !xrat_support::refs::is_ref_prefix(raw) {
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

    /// Resolve a numeric subscription id or ref prefix.
    pub async fn resolve_subscription_id(&self, raw: &str) -> Result<Option<i64>> {
        if let Ok(id) = raw.parse::<i64>()
            && self.repository.get_subscription_by_id(id).await?.is_some()
        {
            return Ok(Some(id));
        }
        if !xrat_support::refs::is_ref_prefix(raw) {
            return Ok(None);
        }
        match self.repository.resolve_subscription_ref_prefix(raw).await? {
            RefMatch::Unique(id) => Ok(Some(id)),
            RefMatch::Ambiguous => Err(AppError::InvalidArgument(format!(
                "subscription ref prefix '{raw}' is ambiguous; provide more characters"
            ))),
            RefMatch::None => Ok(None),
        }
    }

    /// List subscriptions for ref enrichment.
    pub async fn subscriptions(&self) -> Result<Vec<xrat_db::SubscriptionRecord>> {
        Ok(self.repository.list_subscriptions().await?)
    }
}
