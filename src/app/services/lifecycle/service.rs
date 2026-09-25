use std::sync::Arc;

use crate::app::ports::ConfigRepository;
use crate::app::{AppError, Result};
use crate::db::{ConfigRecord, RefMatch, SubscriptionRecord};

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
    pub async fn resolve_config_id(&self, raw: &str) -> Result<i64> {
        if let Ok(id) = raw.parse::<i64>()
            && self.repository.get_config_by_id(id).await?.is_some()
        {
            return Ok(id);
        }

        if crate::support::refs::is_ref_prefix(raw) {
            match self.repository.resolve_config_ref_prefix(raw).await? {
                RefMatch::Unique(id) => return Ok(id),
                RefMatch::Ambiguous => {
                    return Err(AppError::InvalidArgument(format!(
                        "config ref prefix '{raw}' is ambiguous; provide more characters"
                    )));
                }
                RefMatch::None => {}
            }
        }

        Err(AppError::InvalidArgument(format!(
            "no config found for '{raw}'"
        )))
    }

    /// Resolve a numeric id or ref prefix into a subscription id.
    pub async fn resolve_subscription_id(&self, raw: &str) -> Result<i64> {
        if let Ok(id) = raw.parse::<i64>()
            && self.repository.get_subscription_by_id(id).await?.is_some()
        {
            return Ok(id);
        }

        if crate::support::refs::is_ref_prefix(raw) {
            match self.repository.resolve_subscription_ref_prefix(raw).await? {
                RefMatch::Unique(id) => return Ok(id),
                RefMatch::Ambiguous => {
                    return Err(AppError::InvalidArgument(format!(
                        "subscription ref prefix '{raw}' is ambiguous; provide more characters"
                    )));
                }
                RefMatch::None => {}
            }
        }

        Err(AppError::InvalidArgument(format!(
            "no subscription found for '{raw}'"
        )))
    }

    /// Load a config record or report that the identifier matched nothing.
    pub async fn config(&self, raw: &str) -> Result<ConfigRecord> {
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

    pub async fn enable(&self, raw: &str) -> Result<ToggleOutcome> {
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

    pub async fn disable(&self, raw: &str) -> Result<ToggleOutcome> {
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

    pub async fn delete(&self, raw: &str, hard: bool) -> Result<(ConfigRecord, DeleteOutcome)> {
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

    pub async fn restore(&self, raw: &str) -> Result<(ConfigRecord, RestoreOutcome)> {
        let config = self.config(raw).await?;
        if !config.is_deleted {
            return Ok((config, RestoreOutcome::NotDeleted));
        }
        self.repository.restore_config(config.id).await?;
        Ok((config, RestoreOutcome::Restored))
    }
}
