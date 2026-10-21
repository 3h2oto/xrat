use crate::app::ports::ConfigRepository;
use xrat_db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};
use xrat_db::{ConfigRecord, Database, DbError, SubscriptionRecord};
use xrat_model::{ConfigId, SubscriptionId};

/// Adapter that implements [`ConfigRepository`] over the database facade.
///
/// Kept in the application layer so the database module stays unaware of
/// application ports.
#[derive(Clone)]
pub struct DatabaseConfigRepository {
    db: Option<Database>,
}

impl DatabaseConfigRepository {
    pub fn new(db: Database) -> Self {
        Self { db: Some(db) }
    }
    #[cfg(any(windows, test))]
    pub(crate) fn detached() -> Self {
        Self { db: None }
    }

    fn database(&self) -> Result<&Database, DbError> {
        self.db.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "machine catalog persistence is available only through the service gateway",
            )
            .into()
        })
    }
}

#[async_trait::async_trait]
impl ConfigRepository for DatabaseConfigRepository {
    async fn list_configs_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.database()?
            .list_configs_with_latest_tests(filter)
            .await
    }

    async fn list_configs_paginated_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.database()?
            .list_configs_paginated_with_latest_tests(filter, offset, limit)
            .await
    }

    async fn get_config_with_latest_test(
        &self,
        id: ConfigId,
    ) -> Result<Option<ConfigWithLatestTest>, DbError> {
        self.database()?.get_config_with_latest_test(id).await
    }

    async fn count_filtered_configs(&self, filter: &ConfigListFilter) -> Result<i64, DbError> {
        self.database()?.count_filtered_configs(filter).await
    }

    async fn list_top_configs_by_real_delay(
        &self,
        limit: i64,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.database()?
            .list_top_configs_by_real_delay(limit, filter)
            .await
    }

    async fn list_configs(&self, filter: &ConfigListFilter) -> Result<Vec<ConfigRecord>, DbError> {
        self.database()?.list_configs(filter).await
    }

    async fn resolve_config_ref_prefix(&self, prefix: &str) -> Result<RefMatch<ConfigId>, DbError> {
        self.database()?.resolve_config_ref_prefix(prefix).await
    }

    async fn get_config_by_id(&self, id: ConfigId) -> Result<Option<ConfigRecord>, DbError> {
        self.database()?.get_config_by_id(id).await
    }

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionRecord>, DbError> {
        self.database()?.list_subscriptions().await
    }

    async fn resolve_subscription_ref_prefix(
        &self,
        prefix: &str,
    ) -> Result<RefMatch<SubscriptionId>, DbError> {
        self.database()?
            .resolve_subscription_ref_prefix(prefix)
            .await
    }

    async fn get_subscription_by_id(
        &self,
        id: SubscriptionId,
    ) -> Result<Option<SubscriptionRecord>, DbError> {
        self.database()?.get_subscription_by_id(id).await
    }

    async fn set_config_enabled(&self, id: ConfigId, is_enabled: bool) -> Result<(), DbError> {
        self.database()?.set_config_enabled(id, is_enabled).await
    }

    async fn delete_config(&self, id: ConfigId) -> Result<(), DbError> {
        self.database()?.delete_config(id).await
    }

    async fn hard_delete_config(&self, id: ConfigId) -> Result<(), DbError> {
        self.database()?.hard_delete_config(id).await
    }

    async fn restore_config(&self, id: ConfigId) -> Result<(), DbError> {
        self.database()?.restore_config(id).await
    }

    async fn delete_subscription_with_configs(&self, id: SubscriptionId) -> Result<(), DbError> {
        self.database()?.delete_subscription_with_configs(id).await
    }
    async fn set_subscription_name(&self, id: SubscriptionId, name: &str) -> Result<(), DbError> {
        self.database()?.set_subscription_name(id, name).await
    }
    async fn delete_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError> {
        self.database()?.delete_configs(ids).await
    }
    async fn hard_delete_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError> {
        self.database()?.hard_delete_configs(ids).await
    }
    async fn restore_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError> {
        self.database()?.restore_configs(ids).await
    }
    async fn purge_deleted_configs(&self) -> Result<u64, DbError> {
        self.database()?.purge_deleted_configs().await
    }
}
