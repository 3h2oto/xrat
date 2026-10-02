use sqlx::{Postgres, QueryBuilder, Sqlite};

use crate::connection::DbPool;
use xrat_model::ConfigId;

pub async fn clear_all_active(pool: &DbPool) -> crate::Result<()> {
    match pool {
        DbPool::Sqlite(_) => {
            execute_no_bind(
                pool,
                "UPDATE configs SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE is_active = 1",
            )
            .await
        }
        DbPool::Postgres(_) => {
            execute_no_bind(
                pool,
                "UPDATE configs SET is_active = 0, updated_at = CURRENT_TIMESTAMP::TEXT WHERE is_active = 1",
            )
            .await
        }
    }
}

pub async fn mark_active(pool: &DbPool, id: ConfigId) -> crate::Result<()> {
    execute_id(
        pool,
        "UPDATE configs SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        "UPDATE configs SET is_active = 1, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1",
        id,
    )
    .await
}

pub async fn set_enabled(pool: &DbPool, id: ConfigId, enabled: bool) -> crate::Result<()> {
    let enabled_flag = if enabled { 1 } else { 0 };
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE configs SET is_enabled = ?2, is_active = CASE WHEN ?2 = 0 THEN 0 ELSE is_active END, updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(id).bind(enabled_flag).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE configs SET is_enabled = $2, is_active = CASE WHEN $2 = 0 THEN 0 ELSE is_active END, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(id).bind(enabled_flag).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn soft_delete(pool: &DbPool, id: ConfigId) -> crate::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE configs SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP, is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(id).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE configs SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP::TEXT, is_active = 0, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(id).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn restore(pool: &DbPool, id: ConfigId) -> crate::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE configs SET is_deleted = 0, deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(id).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE configs SET is_deleted = 0, deleted_at = NULL, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(id).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn hard_delete(pool: &DbPool, id: ConfigId) -> crate::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut tx = pool.begin().await?;
            sqlx::query("DELETE FROM connection_tests WHERE config_id = ?1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM runtime_sessions WHERE config_id = ?1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM configs WHERE id = ?1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
        DbPool::Postgres(pool) => {
            let mut tx = pool.begin().await?;
            sqlx::query("DELETE FROM connection_tests WHERE config_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM runtime_sessions WHERE config_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM configs WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
    }
    Ok(())
}

pub async fn purge_deleted(pool: &DbPool) -> crate::Result<u64> {
    const SELECT_DELETED: &str = "SELECT id FROM configs WHERE is_deleted = 1";
    let rows_affected = match pool {
        DbPool::Sqlite(pool) => {
            let mut tx = pool.begin().await?;
            sqlx::query(&format!(
                "DELETE FROM connection_tests WHERE config_id IN ({SELECT_DELETED})"
            ))
            .execute(&mut *tx)
            .await?;
            sqlx::query(&format!(
                "DELETE FROM runtime_sessions WHERE config_id IN ({SELECT_DELETED})"
            ))
            .execute(&mut *tx)
            .await?;
            let result = sqlx::query("DELETE FROM configs WHERE is_deleted = 1")
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            result.rows_affected()
        }
        DbPool::Postgres(pool) => {
            let mut tx = pool.begin().await?;
            sqlx::query(&format!(
                "DELETE FROM connection_tests WHERE config_id IN ({SELECT_DELETED})"
            ))
            .execute(&mut *tx)
            .await?;
            sqlx::query(&format!(
                "DELETE FROM runtime_sessions WHERE config_id IN ({SELECT_DELETED})"
            ))
            .execute(&mut *tx)
            .await?;
            let result = sqlx::query("DELETE FROM configs WHERE is_deleted = 1")
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            result.rows_affected()
        }
    };
    Ok(rows_affected)
}

pub async fn soft_delete_many(pool: &DbPool, ids: &[ConfigId]) -> crate::Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let rows_affected = match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "UPDATE configs SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP, is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE id IN (",
            );
            push_id_list(&mut builder, ids);
            builder.push(")");
            builder.build().execute(pool).await?.rows_affected()
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(
                "UPDATE configs SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP::TEXT, is_active = 0, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id IN (",
            );
            push_id_list(&mut builder, ids);
            builder.push(")");
            builder.build().execute(pool).await?.rows_affected()
        }
    };
    Ok(rows_affected)
}

pub async fn restore_many(pool: &DbPool, ids: &[ConfigId]) -> crate::Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let rows_affected = match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "UPDATE configs SET is_deleted = 0, deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id IN (",
            );
            push_id_list(&mut builder, ids);
            builder.push(")");
            builder.build().execute(pool).await?.rows_affected()
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(
                "UPDATE configs SET is_deleted = 0, deleted_at = NULL, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id IN (",
            );
            push_id_list(&mut builder, ids);
            builder.push(")");
            builder.build().execute(pool).await?.rows_affected()
        }
    };
    Ok(rows_affected)
}

pub async fn hard_delete_many(pool: &DbPool, ids: &[ConfigId]) -> crate::Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let rows_affected = match pool {
        DbPool::Sqlite(pool) => {
            let mut tx = pool.begin().await?;
            build_in_delete::<Sqlite>("connection_tests", "config_id", ids)
                .build()
                .execute(&mut *tx)
                .await?;
            build_in_delete::<Sqlite>("runtime_sessions", "config_id", ids)
                .build()
                .execute(&mut *tx)
                .await?;
            let result = build_in_delete::<Sqlite>("configs", "id", ids)
                .build()
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            result.rows_affected()
        }
        DbPool::Postgres(pool) => {
            let mut tx = pool.begin().await?;
            build_in_delete::<Postgres>("connection_tests", "config_id", ids)
                .build()
                .execute(&mut *tx)
                .await?;
            build_in_delete::<Postgres>("runtime_sessions", "config_id", ids)
                .build()
                .execute(&mut *tx)
                .await?;
            let result = build_in_delete::<Postgres>("configs", "id", ids)
                .build()
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            result.rows_affected()
        }
    };
    Ok(rows_affected)
}

fn build_in_delete<'args, DB>(
    table: &str,
    column: &str,
    ids: &'args [ConfigId],
) -> QueryBuilder<'args, DB>
where
    DB: sqlx::Database,
    ConfigId: sqlx::Type<DB> + sqlx::Encode<'args, DB>,
{
    let mut builder = QueryBuilder::<DB>::new(format!("DELETE FROM {table} WHERE {column} IN ("));
    push_id_list(&mut builder, ids);
    builder.push(")");
    builder
}

fn push_id_list<'args, DB>(builder: &mut QueryBuilder<'args, DB>, ids: &'args [ConfigId])
where
    DB: sqlx::Database,
    ConfigId: sqlx::Type<DB> + sqlx::Encode<'args, DB>,
{
    let mut separated = builder.separated(", ");
    for id in ids {
        separated.push_bind(*id);
    }
}

async fn execute_no_bind(pool: &DbPool, sql: &str) -> crate::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query(sql).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query(sql).execute(pool).await?;
        }
    }
    Ok(())
}

async fn execute_id(
    pool: &DbPool,
    sqlite_sql: &str,
    postgres_sql: &str,
    id: ConfigId,
) -> crate::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query(sqlite_sql).bind(id).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query(postgres_sql).bind(id).execute(pool).await?;
        }
    }
    Ok(())
}
