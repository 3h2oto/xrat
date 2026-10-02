use super::super::Database;
use super::super::types::*;

impl Database {
    pub async fn delete_config(&self, id: ConfigId) -> crate::Result<()> {
        repository::delete_config(&self.pool, id).await
    }

    pub async fn restore_config(&self, id: ConfigId) -> crate::Result<()> {
        repository::restore_config(&self.pool, id).await
    }

    pub async fn hard_delete_config(&self, id: ConfigId) -> crate::Result<()> {
        repository::hard_delete_config(&self.pool, id).await
    }

    pub async fn count_deleted_configs(&self) -> crate::Result<i64> {
        repository::count_deleted_configs(&self.pool).await
    }

    pub async fn purge_deleted_configs(&self) -> crate::Result<u64> {
        repository::purge_deleted_configs(&self.pool).await
    }

    pub async fn delete_configs(&self, ids: &[ConfigId]) -> crate::Result<u64> {
        repository::delete_configs(&self.pool, ids).await
    }

    pub async fn restore_configs(&self, ids: &[ConfigId]) -> crate::Result<u64> {
        repository::restore_configs(&self.pool, ids).await
    }

    pub async fn hard_delete_configs(&self, ids: &[ConfigId]) -> crate::Result<u64> {
        repository::hard_delete_configs(&self.pool, ids).await
    }

    pub async fn set_active_config(&self, id: ConfigId) -> crate::Result<()> {
        repository::set_active_config(&self.pool, id).await
    }

    pub async fn clear_active_config(&self) -> crate::Result<()> {
        repository::clear_active_config(&self.pool).await
    }

    pub async fn set_config_enabled(&self, id: ConfigId, is_enabled: bool) -> crate::Result<()> {
        repository::set_config_enabled(&self.pool, id, is_enabled).await
    }
}
