use super::checksum::*;
use super::prelude::*;
use std::collections::HashMap;

/// Reconcile stored migration checksums with the current migration files so that
/// reformatting an already-applied migration does not break the database. The
/// normalized checksum of each applied migration is recorded in
/// `_xrat_migration_norms`; a raw mismatch is healed only when the normalized
/// SQL is unchanged (a formatting edit). A normalized mismatch means the
/// migration's meaning changed, which is rejected.
pub(crate) async fn repair_checksums_sqlite(pool: &SqlitePool) -> crate::Result<()> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    if exists.is_none() {
        return Ok(());
    }

    sqlx::query(NORM_TABLE_SQLITE).execute(pool).await?;
    let applied = sqlx::query(
        "SELECT m.version, m.checksum, n.norm_checksum FROM _sqlx_migrations m \
         LEFT JOIN _xrat_migration_norms n ON n.version = m.version WHERE m.success = 1",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        (
            row.get::<i64, _>("version"),
            (
                row.get::<Vec<u8>, _>("checksum"),
                row.get::<Option<Vec<u8>>, _>("norm_checksum"),
            ),
        )
    })
    .collect::<HashMap<_, _>>();

    for migration in SQLITE_MIGRATOR.iter() {
        if migration.migration_type.is_down_migration() {
            continue;
        }
        let version = migration.version;
        let file_raw = migration.checksum.to_vec();
        let file_norm = normalized_checksum(&migration.sql);

        let Some((applied_checksum, stored_norm)) = applied.get(&version) else {
            continue;
        };

        if *applied_checksum == file_raw {
            if stored_norm.as_deref() != Some(file_norm.as_slice()) {
                record_norm_sqlite(pool, version, &file_norm).await?;
            }
            continue;
        }

        if matches!(stored_norm, Some(norm) if *norm != file_norm) {
            return Err(semantic_change_error("SQLite", version));
        }

        sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
            .bind(file_raw.as_slice())
            .bind(version)
            .execute(pool)
            .await?;
        record_norm_sqlite(pool, version, &file_norm).await?;
    }
    Ok(())
}

pub(crate) async fn record_norm_sqlite(
    pool: &SqlitePool,
    version: i64,
    norm: &[u8],
) -> crate::Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO _xrat_migration_norms (version, norm_checksum) VALUES (?, ?)",
    )
    .bind(version)
    .bind(norm)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn repair_checksums_postgres(pool: &PgPool) -> crate::Result<()> {
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '_sqlx_migrations')",
    )
    .fetch_optional(pool)
    .await?;
    if exists != Some(true) {
        return Ok(());
    }

    sqlx::query(NORM_TABLE_POSTGRES).execute(pool).await?;
    let applied = sqlx::query(
        "SELECT m.version, m.checksum, n.norm_checksum FROM _sqlx_migrations m \
         LEFT JOIN _xrat_migration_norms n ON n.version = m.version WHERE m.success = true",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        (
            row.get::<i64, _>("version"),
            (
                row.get::<Vec<u8>, _>("checksum"),
                row.get::<Option<Vec<u8>>, _>("norm_checksum"),
            ),
        )
    })
    .collect::<HashMap<_, _>>();

    for migration in POSTGRES_MIGRATOR.iter() {
        if migration.migration_type.is_down_migration() {
            continue;
        }
        let version = migration.version;
        let file_raw = migration.checksum.to_vec();
        let file_norm = normalized_checksum(&migration.sql);

        let Some((applied_checksum, stored_norm)) = applied.get(&version) else {
            continue;
        };

        if *applied_checksum == file_raw {
            if stored_norm.as_deref() != Some(file_norm.as_slice()) {
                record_norm_postgres(pool, version, &file_norm).await?;
            }
            continue;
        }

        if matches!(stored_norm, Some(norm) if *norm != file_norm) {
            return Err(semantic_change_error("PostgreSQL", version));
        }

        sqlx::query("UPDATE _sqlx_migrations SET checksum = $1 WHERE version = $2")
            .bind(file_raw.as_slice())
            .bind(version)
            .execute(pool)
            .await?;
        record_norm_postgres(pool, version, &file_norm).await?;
    }
    Ok(())
}

pub(crate) async fn record_norm_postgres(
    pool: &PgPool,
    version: i64,
    norm: &[u8],
) -> crate::Result<()> {
    sqlx::query(
        "INSERT INTO _xrat_migration_norms (version, norm_checksum) VALUES ($1, $2) \
         ON CONFLICT (version) DO UPDATE SET norm_checksum = EXCLUDED.norm_checksum",
    )
    .bind(version)
    .bind(norm)
    .execute(pool)
    .await?;
    Ok(())
}
