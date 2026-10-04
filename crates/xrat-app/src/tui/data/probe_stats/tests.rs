use super::*;

fn record(id: i64, real_delay_ms: Option<i64>, failure: Option<&str>) -> ConnectionTestRecord {
    ConnectionTestRecord {
        id,
        run_id: None,
        config_id: xrat_model::ConfigId(1),
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: None,
        tcp_ms: None,
        real_delay_ok: real_delay_ms.map(|_| true),
        real_delay_ms,
        download_mbps: None,
        upload_mbps: None,
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        dial_endpoint_ip: None,
        dial_endpoint_location: None,
        dial_endpoint_country: None,
        dial_endpoint_asn: None,
        dial_endpoint_geoip_source: None,
        dial_endpoint_fronting: None,
        failure_kind: failure.map(str::to_string),
        failure_reason: None,
        tested_at: format!("2026-06-11T00:00:{id:02}"),
    }
}

#[test]
fn summary_uses_newest_value_and_population_std() {
    // newest-first: 10, 20, 30 -> mean 20, std sqrt(200/3)
    let records = vec![
        record(3, Some(10), None),
        record(2, Some(20), None),
        record(1, Some(30), None),
    ];
    let history = TuiProbeHistory::from_records(&records);
    assert_eq!(history.real_delay.current, Some(10.0));
    assert_eq!(history.real_delay.mean, Some(20.0));
    assert_eq!(history.real_delay.count, 3);
    let std = history.real_delay.std.unwrap();
    assert!((std - (200.0f64 / 3.0).sqrt()).abs() < 1e-9);
}

#[test]
fn missing_values_are_skipped_in_summary() {
    let records = vec![
        record(3, None, Some("timeout")),
        record(2, Some(40), None),
        record(1, Some(60), None),
    ];
    let history = TuiProbeHistory::from_records(&records);
    assert_eq!(history.real_delay.current, Some(40.0));
    assert_eq!(history.real_delay.count, 2);
    assert_eq!(history.real_delay.mean, Some(50.0));
}

#[test]
fn empty_records_yield_default_summary() {
    let history = TuiProbeHistory::from_records(&[]);
    assert_eq!(history.real_delay, MetricSummary::default());
    assert!(history.real_delay_points.is_empty());
    assert_eq!(history.run_count, 0);
    assert!(history.last_tested.is_none());
}

#[test]
fn series_are_chronological_and_totals_tracked() {
    // newest-first input; oldest=id1 -> x=0, newest=id3 -> x=2.
    let records = vec![
        record(3, Some(50), None),
        record(2, None, Some("refused")),
        record(1, Some(30), None),
    ];
    let history = TuiProbeHistory::from_records(&records);
    assert_eq!(history.run_count, 3);
    assert_eq!(history.last_tested.as_deref(), Some("2026-06-11T00:00:03"));
    assert_eq!(history.real_delay_points, vec![(0.0, 30.0), (2.0, 50.0)]);
}
