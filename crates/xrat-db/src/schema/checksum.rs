use super::prelude::*;

pub(crate) fn collision_safe_dedup_key(
    config_id: i64,
    canonical_key: String,
    occupied_by: Option<i64>,
) -> String {
    let Some(existing_id) = occupied_by else {
        return canonical_key;
    };

    tracing::warn!(
        config_id,
        existing_id,
        "preserving legacy config whose canonical dedup key is already occupied"
    );
    format!("v2-legacy|id={config_id}|{canonical_key}")
}

/// Normalize migration SQL so cosmetic reformatting (whitespace, line wrapping,
/// `--` line comments) does not change the value we compare. This assumes
/// migrations do not contain `--` inside a string literal, which holds for this
/// project.
pub(crate) fn normalize_sql(sql: &str) -> String {
    let mut stripped = String::with_capacity(sql.len());
    for line in sql.lines() {
        let code = match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        };
        stripped.push_str(code);
        stripped.push('\n');
    }
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// SQLx derives a migration's checksum from its SQL text alone, so we can
/// recompute the checksum of any SQL string (raw or normalized) the same way.
pub(crate) fn sqlx_checksum(sql: &str) -> Vec<u8> {
    Migration::new(
        0,
        Cow::Borrowed("checksum"),
        MigrationType::Simple,
        Cow::Owned(sql.to_string()),
        false,
    )
    .checksum
    .into_owned()
}

pub(crate) fn normalized_checksum(sql: &str) -> Vec<u8> {
    sqlx_checksum(&normalize_sql(sql))
}

pub(crate) fn semantic_change_error(backend: &str, version: i64) -> DbError {
    DbError::MigrationFailed(format!(
        "{backend} database migration failed: migration {version} was already applied, but its SQL \
         has changed in a way that is not just formatting. Never change the meaning of an applied \
         migration; add a new ordered migration instead. Restore migration {version} to its \
         released form, or reset the database from a backup."
    ))
}

pub(crate) const NORM_TABLE_SQLITE: &str = "CREATE TABLE IF NOT EXISTS _xrat_migration_norms \
     (version INTEGER PRIMARY KEY, norm_checksum BLOB NOT NULL)";

pub(crate) const NORM_TABLE_POSTGRES: &str = "CREATE TABLE IF NOT EXISTS _xrat_migration_norms \
     (version BIGINT PRIMARY KEY, norm_checksum BYTEA NOT NULL)";
