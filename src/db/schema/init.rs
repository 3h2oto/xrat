use super::backfill::*;
use super::error::*;
use super::prelude::*;
use super::repair::*;

pub async fn init_sqlite(pool: &SqlitePool) -> crate::db::Result<()> {
    repair_checksums_sqlite(pool).await?;
    SQLITE_MIGRATOR
        .run(pool)
        .await
        .map_err(|err| migration_error("SQLite", err))?;
    backfill_sqlite_config_extensions(pool).await?;
    Ok(())
}

pub async fn init_postgres(pool: &PgPool) -> crate::db::Result<()> {
    repair_checksums_postgres(pool).await?;
    POSTGRES_MIGRATOR
        .run(pool)
        .await
        .map_err(|err| migration_error("PostgreSQL", err))?;
    backfill_postgres_config_extensions(pool).await?;
    Ok(())
}
