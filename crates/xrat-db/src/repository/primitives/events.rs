use crate::connection::DbPool;
use crate::record::{EventFilter, EventRecord, NewEvent};
use crate::repository::events;

pub async fn record_event(pool: &DbPool, event: &NewEvent) -> crate::Result<i64> {
    events::insert(pool, event).await
}

pub async fn list_events(pool: &DbPool, filter: &EventFilter) -> crate::Result<Vec<EventRecord>> {
    events::query(pool, filter).await
}

pub async fn events_after(
    pool: &DbPool,
    after_id: i64,
    filter: &EventFilter,
) -> crate::Result<Vec<EventRecord>> {
    events::query_after(pool, after_id, filter).await
}

pub async fn clear_events(pool: &DbPool) -> crate::Result<u64> {
    events::delete_all(pool).await
}
