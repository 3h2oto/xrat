use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::record::SubscriptionRecord;

pub fn map_subscription_row<R>(row: R) -> crate::Result<SubscriptionRecord>
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    Ok(SubscriptionRecord {
        id: row.get("id"),
        r#ref: row.get("ref"),
        source_kind: row.get("source_kind"),
        source_url: row.get("source_url"),
        name: row.get("name"),
        headers: serde_json::from_str(&row.get::<String, _>("headers_json"))?,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        config_count: row.get("config_count"),
    })
}
