use crate::connection::DbPool;
use crate::record::{ConnectionTestCursor, ConnectionTestRecord, ConnectionTestSelection};
use crate::repository::row::map_connection_test_row;
use sqlx::{Postgres, QueryBuilder, Sqlite};

pub async fn page(
    pool: &DbPool,
    selection: &ConnectionTestSelection,
    before: Option<&ConnectionTestCursor>,
    limit: i64,
) -> crate::Result<Vec<ConnectionTestRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(query::<Sqlite>(selection, before, limit)
            .build()
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(map_connection_test_row)
            .collect()),
        DbPool::Postgres(pool) => Ok(query::<Postgres>(selection, before, limit)
            .build()
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(map_connection_test_row)
            .collect()),
    }
}

fn query<'args, DB>(
    selection: &ConnectionTestSelection,
    before: Option<&'args ConnectionTestCursor>,
    limit: i64,
) -> QueryBuilder<'args, DB>
where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    let mut query = QueryBuilder::<DB>::new(
        "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE ",
    );
    match selection {
        ConnectionTestSelection::Config(id) => {
            query.push("config_id = ").push_bind(id.0);
        }
        ConnectionTestSelection::Run(id) => {
            query.push("run_id = ").push_bind(*id);
        }
    }
    if let Some(before) = before {
        query
            .push(" AND (tested_at < ")
            .push_bind(before.tested_at.as_str())
            .push(" OR (tested_at = ")
            .push_bind(before.tested_at.as_str())
            .push(" AND id < ")
            .push_bind(before.id)
            .push("))");
    }
    query
        .push(" ORDER BY tested_at DESC, id DESC LIMIT ")
        .push_bind(limit.clamp(1, 200));
    query
}
