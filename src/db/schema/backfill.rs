use super::checksum::*;
use super::prelude::*;

pub(crate) async fn backfill_sqlite_config_extensions(pool: &SqlitePool) -> crate::db::Result<()> {
    let mut transaction = pool.begin().await?;
    let rows =
        sqlx::query("SELECT id, raw_config FROM configs WHERE dedup_key LIKE 'v1%' ORDER BY id")
            .fetch_all(&mut *transaction)
            .await?;
    for row in rows {
        let id: i64 = row.get("id");
        let raw_config: String = row.get("raw_config");
        let Some(node) = crate::config::parse_link(&raw_config).ok().flatten() else {
            continue;
        };
        let extensions_json = node
            .extensions
            .as_ref()
            .and_then(|extensions| serde_json::to_string(extensions).ok());
        let canonical_key = node.dedup_key_string();
        let occupied_by = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM configs WHERE dedup_key = ?1 AND id <> ?2 LIMIT 1",
        )
        .bind(&canonical_key)
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await?;
        let dedup_key = collision_safe_dedup_key(id, canonical_key, occupied_by);
        sqlx::query("UPDATE configs SET dedup_key = ?1, extensions_json = ?2 WHERE id = ?3")
            .bind(dedup_key)
            .bind(extensions_json)
            .bind(id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub(crate) async fn backfill_postgres_config_extensions(pool: &PgPool) -> crate::db::Result<()> {
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        "SELECT id, raw_config FROM configs WHERE dedup_key LIKE 'v1%' ORDER BY id FOR UPDATE",
    )
    .fetch_all(&mut *transaction)
    .await?;
    for row in rows {
        let id: i64 = row.get("id");
        let raw_config: String = row.get("raw_config");
        let Some(node) = crate::config::parse_link(&raw_config).ok().flatten() else {
            continue;
        };
        let extensions_json = node
            .extensions
            .as_ref()
            .and_then(|extensions| serde_json::to_string(extensions).ok());
        let canonical_key = node.dedup_key_string();
        let occupied_by = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM configs WHERE dedup_key = $1 AND id <> $2 LIMIT 1",
        )
        .bind(&canonical_key)
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await?;
        let dedup_key = collision_safe_dedup_key(id, canonical_key, occupied_by);
        sqlx::query("UPDATE configs SET dedup_key = $1, extensions_json = $2 WHERE id = $3")
            .bind(dedup_key)
            .bind(extensions_json)
            .bind(id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}
