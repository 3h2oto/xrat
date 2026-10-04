#[cfg(test)]
pub(super) use std::path::PathBuf;
#[cfg(test)]
pub(super) use std::time::Duration;
#[cfg(test)]
pub(super) use std::time::{SystemTime, UNIX_EPOCH};

pub(super) use crate::connection::{self, DatabaseConnectionConfig, DbPool};
pub(super) use crate::record::{
    CfScanResultRecord, CfScanResultUpsert, ConfigListFilter, ConfigRecord, ConfigWithLatestTest,
    ConnectionTestInsert, ConnectionTestRecord, ConnectionTestRunInsert, ConnectionTestRunRecord,
    EventFilter, EventRecord, GeoIpCacheRecord, GeoIpCacheUpsert, ImportSource, ImportSummary,
    NewEvent, RefMatch, RefreshableSubscription, RuntimeSessionInsert, RuntimeSessionRecord,
    RuntimeSessionStatus, SubscriptionRecord,
};
pub(super) use crate::repository;
pub(super) use xrat_model::{ConfigId, SubscriptionId};

#[derive(Clone)]
pub struct Database {
    pub(super) pool: DbPool,
}
