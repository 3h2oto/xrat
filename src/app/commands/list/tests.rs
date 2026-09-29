use super::configs::*;
use super::prelude::*;
use super::subscriptions::*;

#[test]
fn config_outputs_include_refs() {
    let config = config_row("abcdef123456");
    let subscriptions = HashMap::from([(2, "123456abcdef")]);

    let table = format_config_table(std::slice::from_ref(&config), &subscriptions, None);
    let tsv = format_config_tsv(std::slice::from_ref(&config), &subscriptions);
    let json = config_json(&config, &subscriptions);

    assert!(table.contains("REF"));
    assert!(table.contains("ICMP"));
    assert!(table.contains("42ms"));
    assert!(table.contains("COUNTRY"));
    assert!(table.contains("NL"));
    assert!(table.contains("abcdef12"));
    assert!(table.contains("123456ab"));
    assert!(tsv.starts_with("ref\tsubscription_ref\tstatus\tprotocol\taddress\tport\ticmp_ms\t"));
    assert!(!tsv.starts_with("ref\tid\t"));
    assert!(tsv.contains("\t42\t20\t100\t25.50\t5.75\tNL\tNL/Amsterdam\tAS60781 LeaseWeb\t"));
    assert_eq!(json["ref"], "abcdef123456");
    assert_eq!(json["subscription_ref"], "123456abcdef");
    assert_eq!(json["latest_test"]["icmp_ms"], 42);
    assert_eq!(json["latest_test"]["dial_endpoint_country"], "NL");
    assert!(json.get("id").is_none());
    assert!(json.get("subscription_id").is_none());
}

#[test]
fn subscription_outputs_include_refs() {
    let subscription = SubscriptionRecord {
        id: 2,
        r#ref: "123456abcdef".to_string(),
        source_kind: "url".to_string(),
        source_url: Some("https://example.com/sub".to_string()),
        name: Some("main".to_string()),
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
        config_count: 3,
    };

    let table = format_subscription_table(std::slice::from_ref(&subscription));
    let tsv = format_subscription_tsv(std::slice::from_ref(&subscription));
    let json = subscription_json(&subscription);

    assert!(table.contains("REF"));
    assert!(table.contains("123456ab"));
    assert!(table.contains("UPDATED AT"));
    assert!(table.contains("updated"));
    assert!(tsv.starts_with("ref\tkind\t"));
    assert_eq!(json["ref"], "123456abcdef");
    assert!(json.get("id").is_none());
}

#[test]
fn config_table_uses_enabled_settings_for_metric_columns() {
    let config = config_row("abcdef123456");
    let subscriptions = HashMap::from([(2, "123456abcdef")]);
    let mut settings = crate::app::config::TestingSettings::default();
    settings.icmp.enabled = false;
    settings.tcp.enabled = true;
    settings.real_delay.enabled = true;
    settings.download.enabled = false;
    settings.geoip.enabled = false;

    let table = format_config_table(
        std::slice::from_ref(&config),
        &subscriptions,
        Some(&settings),
    );

    assert!(!table.contains("ICMP"));
    assert!(table.contains("TCP"));
    assert!(table.contains("REAL"));
    assert!(!table.contains("DOWN"));
    assert!(!table.contains("COUNTRY"));
}

fn config_row(value_ref: &str) -> ConfigWithLatestTest {
    ConfigWithLatestTest {
        config: ConfigRecord {
            id: 1,
            r#ref: value_ref.to_string(),
            subscription_id: Some(2),
            dedup_key: "key".to_string(),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: None,
            host: None,
            path: None,
            name: Some("main".to_string()),
            raw_config: "vless://uuid@example.com:443#main".to_string(),
            extensions_json: None,
            is_active: true,
            is_enabled: true,
            is_deleted: false,
            deleted_at: None,
            imported_at: "imported".to_string(),
            created_at: "created".to_string(),
            updated_at: "updated".to_string(),
        },
        test_id: Some(9),
        icmp_ok: Some(true),
        icmp_ms: Some(42),
        tcp_ok: Some(true),
        tcp_ms: Some(20),
        real_delay_ok: Some(true),
        real_delay_ms: Some(100),
        download_mbps: Some(25.5),
        upload_mbps: Some(5.75),
        connect_ms: Some(20),
        ttfb_ms: Some(80),
        http_status: Some(204),
        dial_endpoint_location: Some("NL/Amsterdam".to_string()),
        dial_endpoint_country: Some("NL".to_string()),
        dial_endpoint_asn: Some("AS60781 LeaseWeb".to_string()),
        dial_endpoint_geoip_source: None,
        dial_endpoint_fronting: None,
        failure_kind: None,
        failure_reason: None,
        tested_at: Some("tested".to_string()),
    }
}
