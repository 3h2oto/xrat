use crate::connection::DbPool;
use crate::record::{
    ConnectionTestInsert, ConnectionTestRecord, ConnectionTestRunInsert, ConnectionTestRunRecord,
};
use crate::repository::connection_tests;
use xrat_model::ConfigId;

pub async fn get_connection_test_count(pool: &DbPool) -> crate::Result<i64> {
    connection_tests::get_count(pool).await
}

pub async fn insert_connection_test(
    pool: &DbPool,
    test: &ConnectionTestInsert,
) -> crate::Result<i64> {
    connection_tests::insert(pool, test).await
}

pub async fn insert_connection_test_run(
    pool: &DbPool,
    run: &ConnectionTestRunInsert,
) -> crate::Result<i64> {
    connection_tests::insert_run(pool, run).await
}

pub async fn list_connection_tests(
    pool: &DbPool,
    config_id: ConfigId,
) -> crate::Result<Vec<ConnectionTestRecord>> {
    connection_tests::list_by_config(pool, config_id).await
}

pub async fn get_latest_connection_test(
    pool: &DbPool,
    config_id: ConfigId,
) -> crate::Result<Option<ConnectionTestRecord>> {
    connection_tests::get_latest_by_config(pool, config_id).await
}

pub async fn get_latest_connection_test_run(
    pool: &DbPool,
) -> crate::Result<Option<ConnectionTestRunRecord>> {
    connection_tests::get_latest_run(pool).await
}

pub async fn list_connection_tests_by_run(
    pool: &DbPool,
    run_id: i64,
) -> crate::Result<Vec<ConnectionTestRecord>> {
    connection_tests::list_by_run(pool, run_id).await
}

pub async fn recent_connection_tests(
    pool: &DbPool,
    config_id: ConfigId,
    limit: i64,
) -> crate::Result<Vec<ConnectionTestRecord>> {
    connection_tests::list_recent_by_config(pool, config_id, limit).await
}

pub async fn connection_test_page(
    pool: &DbPool,
    selection: &crate::ConnectionTestSelection,
    before: Option<&crate::ConnectionTestCursor>,
    limit: i64,
) -> crate::Result<Vec<ConnectionTestRecord>> {
    connection_tests::page(pool, selection, before, limit).await
}
