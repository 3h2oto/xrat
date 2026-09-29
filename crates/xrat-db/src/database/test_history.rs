use super::Database;
use super::types::*;

impl Database {
    pub async fn upsert_cf_scan_results(
        &self,
        results: &[CfScanResultUpsert],
    ) -> crate::Result<()> {
        repository::upsert_cf_scan_results(&self.pool, results).await
    }

    pub async fn list_cf_scan_results(&self) -> crate::Result<Vec<CfScanResultRecord>> {
        repository::list_cf_scan_results(&self.pool).await
    }

    pub async fn list_cf_scan_history(&self, limit: i64) -> crate::Result<Vec<CfScanResultRecord>> {
        repository::list_cf_scan_history(&self.pool, limit).await
    }

    pub async fn insert_connection_test(&self, test: &ConnectionTestInsert) -> crate::Result<i64> {
        repository::insert_connection_test(&self.pool, test).await
    }

    pub async fn insert_connection_test_run(
        &self,
        run: &ConnectionTestRunInsert,
    ) -> crate::Result<i64> {
        repository::insert_connection_test_run(&self.pool, run).await
    }

    pub async fn list_connection_tests(
        &self,
        config_id: i64,
    ) -> crate::Result<Vec<ConnectionTestRecord>> {
        repository::list_connection_tests(&self.pool, config_id).await
    }

    pub async fn get_latest_connection_test(
        &self,
        config_id: i64,
    ) -> crate::Result<Option<ConnectionTestRecord>> {
        repository::get_latest_connection_test(&self.pool, config_id).await
    }

    pub async fn get_latest_connection_test_run(
        &self,
    ) -> crate::Result<Option<ConnectionTestRunRecord>> {
        repository::get_latest_connection_test_run(&self.pool).await
    }

    pub async fn list_connection_tests_by_run(
        &self,
        run_id: i64,
    ) -> crate::Result<Vec<ConnectionTestRecord>> {
        repository::list_connection_tests_by_run(&self.pool, run_id).await
    }
}
