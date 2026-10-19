use xrat_web::types::*;

pub(crate) fn config(detail: crate::app::read_models::ConfigDetail) -> ConfigRow {
    let row = detail.summary;
    ConfigRow {
        id: row.id.0,
        reference: row.r#ref,
        name: row.name.unwrap_or_default(),
        protocol: row.protocol,
        address: row.address,
        port: row.port,
        enabled: row.is_enabled,
        deleted: row.is_deleted,
        delay_ms: row.latest_test.as_ref().and_then(|test| test.real_delay_ms),
        subscription_id: row.subscription_id.map(|id| id.0),
        network: row.network,
        tls: row.tls,
        sni: detail.sni,
        country: detail.endpoint_location.country,
        asn: detail.endpoint_location.asn,
        location: detail.endpoint_location.location,
        fronting: detail.endpoint_location.fronting,
        updated_at: detail.updated_at,
        test: row.latest_test.map(|test| ConfigTest {
            icmp_ok: test.icmp_ok,
            icmp_ms: test.icmp_ms,
            tcp_ok: test.tcp_ok,
            tcp_ms: test.tcp_ms,
            real_delay_ok: test.real_delay_ok,
            real_delay_ms: test.real_delay_ms,
            download_mbps: test.download_mbps,
            upload_mbps: test.upload_mbps,
            connect_ms: test.connect_ms,
            ttfb_ms: test.ttfb_ms,
            http_status: test.http_status,
            failure_reason: test.failure_reason,
            tested_at: test.tested_at,
        }),
    }
}

pub(crate) fn subscription(row: xrat_db::SubscriptionRecord) -> SubscriptionRow {
    SubscriptionRow {
        id: row.id.0,
        name: row.name.unwrap_or_else(|| row.r#ref.clone()),
        reference: row.r#ref,
        source: row.source_kind,
        config_count: row.config_count,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
