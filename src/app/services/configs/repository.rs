use crate::app::ports::ConfigRepository;
use crate::db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};
use crate::db::{ConfigRecord, Database, DbError, SubscriptionRecord};

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

    async fn resolve_config_ref_prefix(&self, prefix: &str) -> Result<RefMatch, DbError> {
        self.db.resolve_config_ref_prefix(prefix).await
    }

    async fn get_config_by_id(&self, id: i64) -> Result<Option<ConfigRecord>, DbError> {
        self.db.get_config_by_id(id).await
    }

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionRecord>, DbError> {
        self.db.list_subscriptions().await
    }
}
