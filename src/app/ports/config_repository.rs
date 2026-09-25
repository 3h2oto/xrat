use crate::db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};
use crate::db::{DbError, SubscriptionRecord};

/// Persistence operations required by the config read service.
///
/// This is deliberately narrow: only queries used by `ConfigService`. It is
/// implemented by the database layer and replaced by fakes in tests.
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
        id: i64,
    ) -> Result<Option<ConfigWithLatestTest>, DbError>;

    async fn count_filtered_configs(&self, filter: &ConfigListFilter) -> Result<i64, DbError>;

    async fn resolve_config_ref_prefix(&self, prefix: &str) -> Result<RefMatch, DbError>;

    async fn get_config_by_id(&self, id: i64) -> Result<Option<crate::db::ConfigRecord>, DbError>;

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionRecord>, DbError>;
}
