use xrat_db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};
use xrat_db::{DbError, SubscriptionRecord};
use xrat_model::{ConfigId, SubscriptionId};

/// Persistence operations required by config read and lifecycle services.
///
/// It is implemented by the database layer and replaced by fakes in tests.
#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait::async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn list_configs_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError>;

    async fn list_configs_paginated_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError>;

    async fn get_config_with_latest_test(
        &self,
        id: ConfigId,
    ) -> Result<Option<ConfigWithLatestTest>, DbError>;

    async fn count_filtered_configs(&self, filter: &ConfigListFilter) -> Result<i64, DbError>;

    async fn list_top_configs_by_real_delay(
        &self,
        limit: i64,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError>;

    async fn list_configs(
        &self,
        filter: &ConfigListFilter,
    ) -> Result<Vec<xrat_db::ConfigRecord>, DbError>;

    async fn resolve_config_ref_prefix(&self, prefix: &str) -> Result<RefMatch<ConfigId>, DbError>;

    async fn get_config_by_id(
        &self,
        id: ConfigId,
    ) -> Result<Option<xrat_db::ConfigRecord>, DbError>;

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionRecord>, DbError>;

    async fn resolve_subscription_ref_prefix(
        &self,
        prefix: &str,
    ) -> Result<RefMatch<SubscriptionId>, DbError>;

    async fn get_subscription_by_id(
        &self,
        id: SubscriptionId,
    ) -> Result<Option<SubscriptionRecord>, DbError>;

    async fn set_config_enabled(&self, id: ConfigId, is_enabled: bool) -> Result<(), DbError>;

    async fn delete_config(&self, id: ConfigId) -> Result<(), DbError>;

    async fn hard_delete_config(&self, id: ConfigId) -> Result<(), DbError>;

    async fn restore_config(&self, id: ConfigId) -> Result<(), DbError>;

    async fn delete_subscription_with_configs(&self, id: SubscriptionId) -> Result<(), DbError>;
    async fn set_subscription_name(&self, id: SubscriptionId, name: &str) -> Result<(), DbError>;
    async fn delete_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError>;
    async fn hard_delete_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError>;
    async fn restore_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError>;
}
