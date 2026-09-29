pub(crate) use std::borrow::Cow;

pub(crate) use sqlx::migrate::MigrateError;
pub(crate) use sqlx::migrate::{Migration, MigrationType};
pub(crate) use sqlx::{PgPool, Row, SqlitePool};

pub(crate) use crate::db::DbError;

pub(crate) static SQLITE_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
pub(crate) static POSTGRES_MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("./migrations/postgres");
