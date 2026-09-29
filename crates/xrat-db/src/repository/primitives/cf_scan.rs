use crate::connection::DbPool;
use crate::record::{CfScanResultRecord, CfScanResultUpsert};
use crate::repository::cf_scan_results;

pub async fn upsert_cf_scan_results(
    pool: &DbPool,
    results: &[CfScanResultUpsert],
) -> crate::Result<()> {
    cf_scan_results::upsert_batch(pool, results).await
}

pub async fn list_cf_scan_results(pool: &DbPool) -> crate::Result<Vec<CfScanResultRecord>> {
    cf_scan_results::list_all(pool).await
}

pub async fn list_cf_scan_history(
    pool: &DbPool,
    limit: i64,
) -> crate::Result<Vec<CfScanResultRecord>> {
    cf_scan_results::list_history(pool, limit).await
}
