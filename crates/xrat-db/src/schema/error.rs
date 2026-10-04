use super::prelude::*;

/// Translate a raw sqlx migration failure into an actionable error. Migrations
/// run lazily on the first command after an upgrade, so a bare sqlx message here
/// is easy to misread as unrelated corruption. We attach the migration version,
/// the likely cause, and concrete recovery guidance.
pub(crate) fn migration_error(backend: &str, err: MigrateError) -> DbError {
    let guidance = match &err {
        MigrateError::VersionMismatch(version) => format!(
            "migration {version} was already applied but its checksum no longer matches this build. \
             A previously shipped migration file was most likely edited after release. \
             Policy: never edit an applied migration; always add a new ordered migration. \
             To recover, restore the original {version} migration (e.g. reinstall the matching \
             release) or reset the database from a backup."
        ),
        MigrateError::Dirty(version) => format!(
            "migration {version} is partially applied (dirty state). \
             Inspect the `_sqlx_migrations` table, finish or revert migration {version} by hand, \
             and remove its row before retrying."
        ),
        MigrateError::VersionMissing(version) => format!(
            "migration {version} is recorded as applied in the database but is missing from this build. \
             This usually means a downgrade to an older xrat. Upgrade back to a build that includes \
             migration {version}."
        ),
        MigrateError::ExecuteMigration(source, version) => format!(
            "migration {version} failed while executing against existing data: {source}. \
             Review migration {version} and the current database contents."
        ),
        other => format!("{other}"),
    };

    DbError::MigrationFailed(format!("{backend} database migration failed: {guidance}"))
}
