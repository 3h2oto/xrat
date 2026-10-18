use sqlx::{Postgres, QueryBuilder, Sqlite};
use xrat_model::{Node, SubscriptionId};

const DEPENDENTS: [(&str, &str); 3] = [
    ("connection_tests", "config_id"),
    ("runtime_sessions", "config_id"),
    ("configs", "id"),
];

pub(super) async fn sqlite(
    connection: &mut sqlx::SqliteConnection,
    subscription: SubscriptionId,
    nodes: &[Node],
) -> crate::Result<u64> {
    let removed = mark_missing::<Sqlite>(subscription, nodes, "CURRENT_TIMESTAMP")
        .build()
        .execute(&mut *connection)
        .await?
        .rows_affected();
    for (table, column) in DEPENDENTS {
        purge_missing::<Sqlite>(table, column, subscription, nodes)
            .build()
            .execute(&mut *connection)
            .await?;
    }
    Ok(removed)
}

pub(super) async fn postgres(
    connection: &mut sqlx::PgConnection,
    subscription: SubscriptionId,
    nodes: &[Node],
) -> crate::Result<u64> {
    let removed = mark_missing::<Postgres>(subscription, nodes, "CURRENT_TIMESTAMP::TEXT")
        .build()
        .execute(&mut *connection)
        .await?
        .rows_affected();
    for (table, column) in DEPENDENTS {
        purge_missing::<Postgres>(table, column, subscription, nodes)
            .build()
            .execute(&mut *connection)
            .await?;
    }
    Ok(removed)
}

fn mark_missing<'args, DB>(
    subscription: SubscriptionId,
    nodes: &'args [Node],
    timestamp: &str,
) -> QueryBuilder<'args, DB>
where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    let mut query = QueryBuilder::new(format!(
        "UPDATE configs SET is_deleted = 1, deleted_at = {timestamp} WHERE is_deleted = 0 AND subscription_id = "
    ));
    missing_predicate(&mut query, subscription, nodes);
    query
}

fn purge_missing<'args, DB>(
    table: &str,
    column: &str,
    subscription: SubscriptionId,
    nodes: &'args [Node],
) -> QueryBuilder<'args, DB>
where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    let mut query = QueryBuilder::new(format!(
        "DELETE FROM {table} WHERE {column} IN (SELECT configs.id FROM configs WHERE subscription_id = "
    ));
    missing_predicate(&mut query, subscription, nodes);
    query.push(" AND configs.is_active = 0 AND NOT EXISTS (SELECT 1 FROM runtime_sessions WHERE runtime_sessions.config_id = configs.id AND runtime_sessions.status IN ('starting', 'running', 'stopping')))");
    query
}

fn missing_predicate<'args, DB>(
    query: &mut QueryBuilder<'args, DB>,
    subscription: SubscriptionId,
    nodes: &'args [Node],
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    query
        .push_bind(subscription)
        .push(" AND dedup_key NOT IN (");
    let mut values = query.separated(", ");
    for node in nodes {
        values.push_bind(node.dedup_key_string());
    }
    values.push_unseparated(")");
}
