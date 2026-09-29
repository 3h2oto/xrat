use crate::connection::DbPool;
use crate::record::{RefMatch, RefreshableSubscription, SubscriptionRecord};
use crate::repository::subscriptions;

pub async fn resolve_subscription_ref_prefix(
    pool: &DbPool,
    prefix: &str,
) -> crate::Result<RefMatch> {
    subscriptions::resolve_ref_prefix(pool, prefix).await
}

pub async fn get_subscription_count(pool: &DbPool) -> crate::Result<i64> {
    subscriptions::get_count(pool).await
}

pub async fn list_refreshable_due_subscriptions(
    pool: &DbPool,
    cutoff_epoch_secs: i64,
) -> crate::Result<Vec<RefreshableSubscription>> {
    subscriptions::list_refreshable_due(pool, cutoff_epoch_secs).await
}

pub async fn list_subscriptions(pool: &DbPool) -> crate::Result<Vec<SubscriptionRecord>> {
    subscriptions::list(pool).await
}

pub async fn get_subscription_by_id(
    pool: &DbPool,
    id: i64,
) -> crate::Result<Option<SubscriptionRecord>> {
    subscriptions::get_by_id(pool, id).await
}

pub async fn set_subscription_name(pool: &DbPool, id: i64, name: &str) -> crate::Result<()> {
    subscriptions::set_name(pool, id, name).await
}

pub async fn delete_subscription_with_configs(pool: &DbPool, id: i64) -> crate::Result<()> {
    subscriptions::delete_with_configs(pool, id).await
}
