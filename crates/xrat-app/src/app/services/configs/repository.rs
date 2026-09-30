use crate::app::ports::ConfigRepository;
use xrat_db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};
use xrat_db::{ConfigRecord, Database, DbError, SubscriptionRecord};

/// Adapter that implements [`ConfigRepository`] over the database facade.
///
/// Kept in the application layer so the database module stays unaware of
/// application ports.
#[derive(Clone)]
pub struct DatabaseConfigRepository {
    db: Database,
}

impl DatabaseConfigRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ConfigRepository for DatabaseConfigRepository {
    async fn list_configs_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.db.list_configs_with_latest_tests(filter).await
    }

    async fn list_configs_paginated_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.db
            .list_configs_paginated_with_latest_tests(filter, offset, limit)
            .await
    }

    async fn get_config_with_latest_test(
        &self,
        id: i64,
    ) -> Result<Option<ConfigWithLatestTest>, DbError> {
        self.db.get_config_with_latest_test(id).await
    }

    async fn count_filtered_configs(&self, filter: &ConfigListFilter) -> Result<i64, DbError> {
        self.db.count_filtered_configs(filter).await
    }

    async fn list_top_configs_by_real_delay(
        &self,
        limit: i64,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.db.list_top_configs_by_real_delay(limit, filter).await
    }

    async fn list_configs(&self, filter: &ConfigListFilter) -> Result<Vec<ConfigRecord>, DbError> {
        self.db.list_configs(filter).await
    }

    async fn resolve_config_ref_prefix(&self, prefix: &str) -> Result<RefMatch, DbError> {
        self.db.resolve_config_ref_prefix(prefix).await
    }

    async fn get_config_by_id(&self, id: i64) -> Result<Option<ConfigRecord>, DbError> {
        self.db.get_config_by_id(id).await
    }

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionRecord>, DbError> {
        self.db.list_subscriptions().await
    }

    async fn resolve_subscription_ref_prefix(&self, prefix: &str) -> Result<RefMatch, DbError> {
        self.db.resolve_subscription_ref_prefix(prefix).await
    }

    async fn get_subscription_by_id(&self, id: i64) -> Result<Option<SubscriptionRecord>, DbError> {
        self.db.get_subscription_by_id(id).await
    }

    async fn set_config_enabled(&self, id: i64, is_enabled: bool) -> Result<(), DbError> {
        self.db.set_config_enabled(id, is_enabled).await
    }

    async fn delete_config(&self, id: i64) -> Result<(), DbError> {
        self.db.delete_config(id).await
    }

    async fn hard_delete_config(&self, id: i64) -> Result<(), DbError> {
        self.db.hard_delete_config(id).await
    }

    async fn restore_config(&self, id: i64) -> Result<(), DbError> {
        self.db.restore_config(id).await
    }

    async fn delete_subscription_with_configs(&self, id: i64) -> Result<(), DbError> {
        self.db.delete_subscription_with_configs(id).await
    }
    async fn set_subscription_name(&self, id: i64, name: &str) -> Result<(), DbError> {
        self.db.set_subscription_name(id, name).await
    }
    async fn delete_configs(&self, ids: &[i64]) -> Result<u64, DbError> {
        self.db.delete_configs(ids).await
    }
    async fn hard_delete_configs(&self, ids: &[i64]) -> Result<u64, DbError> {
        self.db.hard_delete_configs(ids).await
    }
    async fn restore_configs(&self, ids: &[i64]) -> Result<u64, DbError> {
        self.db.restore_configs(ids).await
    }
}
