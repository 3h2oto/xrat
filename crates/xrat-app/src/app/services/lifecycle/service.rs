use std::sync::Arc;

use crate::app::ports::ConfigRepository;
use crate::app::{AppError, Result};
use xrat_db::{ConfigRecord, SubscriptionRecord};
use xrat_model::{ConfigId, ConfigRef, SubscriptionId};

use crate::app::services::configs::DatabaseConfigRepository;

/// Outcome of an enable or disable request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToggleOutcome {
    Changed,
    AlreadyEnabled,
    AlreadyDisabled,
    DeletedConfig,
}

/// Outcome of a delete request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeleteOutcome {
    SoftDeleted,
    HardDeleted,
    AlreadyDeleted,
}

/// Outcome of a restore request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreOutcome {
    Restored,
    NotDeleted,
}

/// Config and subscription lifecycle transitions shared by every adapter.
///
/// Methods resolve state, apply rules, and perform persistence. They never
/// print; adapters render outcomes and request confirmation.
#[derive(Clone)]
pub struct ConfigLifecycleService {
    repository: Arc<DatabaseConfigRepository>,
}

impl ConfigLifecycleService {
    pub fn new(repository: Arc<DatabaseConfigRepository>) -> Self {
        Self { repository }
    }

    /// Resolve a numeric id or ref prefix into a config id, or error when it
    /// does not identify a stored config.
    pub async fn resolve_config_id(&self, raw: &ConfigRef) -> Result<ConfigId> {
        crate::app::services::ConfigService::new(Arc::clone(&self.repository))
            .resolve_id(raw)
            .await?
            .ok_or_else(|| AppError::InvalidArgument(format!("no config found for '{raw}'")))
    }

    pub async fn resolve_subscription_id(&self, raw: &str) -> Result<SubscriptionId> {
        crate::app::services::ConfigService::new(Arc::clone(&self.repository))
            .resolve_subscription_id(raw)
            .await?
            .ok_or_else(|| AppError::InvalidArgument(format!("no subscription found for '{raw}'")))
    }

    /// Load a config record or report that the identifier matched nothing.
    pub async fn config(&self, raw: &ConfigRef) -> Result<ConfigRecord> {
        let id = self.resolve_config_id(raw).await?;
        self.repository
            .get_config_by_id(id)
            .await?
            .ok_or_else(|| AppError::InvalidArgument(format!("config {raw} not found")))
    }

    /// Load a subscription record or report that the identifier matched nothing.
    pub async fn subscription(&self, raw: &str) -> Result<SubscriptionRecord> {
        let id = self.resolve_subscription_id(raw).await?;
        self.repository
            .get_subscription_by_id(id)
            .await?
            .ok_or_else(|| AppError::InvalidArgument(format!("subscription {raw} not found")))
    }

    pub async fn enable(&self, raw: &ConfigRef) -> Result<ToggleOutcome> {
        let config = self.config(raw).await?;
        if config.is_deleted {
            return Ok(ToggleOutcome::DeletedConfig);
        }
        if config.is_enabled {
            return Ok(ToggleOutcome::AlreadyEnabled);
        }
        self.repository.set_config_enabled(config.id, true).await?;
        Ok(ToggleOutcome::Changed)
    }

    pub async fn disable(&self, raw: &ConfigRef) -> Result<ToggleOutcome> {
        let config = self.config(raw).await?;
        if config.is_deleted {
            return Ok(ToggleOutcome::DeletedConfig);
        }
        if !config.is_enabled {
            return Ok(ToggleOutcome::AlreadyDisabled);
        }
        self.repository.set_config_enabled(config.id, false).await?;
        Ok(ToggleOutcome::Changed)
    }

    pub async fn delete(
        &self,
        raw: &ConfigRef,
        hard: bool,
    ) -> Result<(ConfigRecord, DeleteOutcome)> {
        let config = self.config(raw).await?;
        if hard {
            self.repository.hard_delete_config(config.id).await?;
            return Ok((config, DeleteOutcome::HardDeleted));
        }
        if config.is_deleted {
            return Ok((config, DeleteOutcome::AlreadyDeleted));
        }
        self.repository.delete_config(config.id).await?;
        Ok((config, DeleteOutcome::SoftDeleted))
    }

    pub async fn delete_subscription(&self, raw: &str) -> Result<SubscriptionRecord> {
        let subscription = self.subscription(raw).await?;
        self.repository
            .delete_subscription_with_configs(subscription.id)
            .await?;
        Ok(subscription)
    }

    pub async fn rename_subscription(&self, raw: &str, name: &str) -> Result<SubscriptionRecord> {
        let subscription = self.subscription(raw).await?;
        self.repository
            .set_subscription_name(subscription.id, name)
            .await?;
        Ok(subscription)
    }

    pub async fn restore(&self, raw: &ConfigRef) -> Result<(ConfigRecord, RestoreOutcome)> {
        let config = self.config(raw).await?;
        if !config.is_deleted {
            return Ok((config, RestoreOutcome::NotDeleted));
        }
        self.repository.restore_config(config.id).await?;
        Ok((config, RestoreOutcome::Restored))
    }
    pub async fn delete_many(&self, ids: &[ConfigId], hard: bool) -> Result<u64> {
        Ok(if hard {
            self.repository.hard_delete_configs(ids).await?
        } else {
            self.repository.delete_configs(ids).await?
        })
    }

    pub async fn restore_many(&self, ids: &[ConfigId]) -> Result<u64> {
        Ok(self.repository.restore_configs(ids).await?)
    }
}
