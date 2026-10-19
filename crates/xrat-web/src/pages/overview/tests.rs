use super::components::{
    charts::{ChartPoint, PlotBounds, paths},
    stats::*,
};
use xrat_web::types::{ConfigRow, ConfigTest, DashboardData, LatencyMetric, Traffic};

fn meta(session: i64, at: f64) -> DashboardData {
    DashboardData {
        sampled_at_ms: at,
        session_id: Some(session),
        runtime_status: "running".into(),
        stats_enabled: true,
        tun_enabled: false,
        tun_active: false,
        tun_available: true,
        tun_reason: None,
        enabled_metrics: METRICS.to_vec(),
        last_subscription_refresh: None,
        probes: vec![],
        traffic_history: vec![],
    }
}
fn traffic(down: u64, up: u64) -> Traffic {
    Traffic {
        downloaded_bytes: down,
        uploaded_bytes: up,
    }
}
fn config(ok: Option<bool>, ms: Option<i64>) -> ConfigRow {
    ConfigRow {
        enabled: true,
        test: Some(ConfigTest {
            tcp_ok: ok,
            tcp_ms: ms,
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn health_excludes_disabled_deleted_and_failed_negative_values_from_average() {
    let mut deleted = config(Some(true), Some(9999));
    deleted.deleted = true;
    let mut disabled = config(Some(true), Some(9999));
    disabled.enabled = false;
    let result = health(
        &[
            config(Some(true), Some(0)),
            config(Some(true), Some(100)),
            config(Some(false), Some(1000)),
            config(None, Some(40)),
            config(Some(true), Some(-1)),
            disabled,
            deleted,
        ],
        LatencyMetric::Tcp,
    );
    assert_eq!((result.passed, result.failed, result.untested), (3, 1, 1));
    assert_eq!(result.average, Some(50.0));
    assert_eq!(result.samples, 2);
}

#[test]
fn throughput_uses_elapsed_seconds_and_requires_two_samples() {
    let mut history = TrafficHistory::default();
    history.record(&meta(1, 1000.0), Some(&traffic(100, 10)), true);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(1, 4000.0), Some(&traffic(1000, 310)), true);
    assert_eq!(history.latest().unwrap().down, Some(300.0));
    assert_eq!(history.latest().unwrap().up, Some(100.0));
    history.record(&meta(1, 4000.0), Some(&traffic(1000, 310)), true);
    assert_eq!(history.points.len(), 2);
    history.interrupt();
    history.record(&meta(1, 7000.0), Some(&traffic(9000, 9000)), true);
    assert_eq!(history.latest().unwrap().down, None);
}

#[test]
fn session_and_counter_resets_do_not_create_spikes_or_carry_old_totals() {
    let mut history = TrafficHistory::default();
    history.record(&meta(1, 1000.0), Some(&traffic(1000, 1000)), true);
    history.record(&meta(1, 4000.0), Some(&traffic(2000, 2000)), true);
    history.record(&meta(2, 7000.0), Some(&traffic(100, 100)), true);
    assert_eq!(history.points.len(), 1);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(2, 10000.0), Some(&traffic(50, 50)), true);
    assert_eq!(history.points.len(), 1);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(2, 13000.0), Some(&traffic(350, 50)), true);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(2, 16000.0), Some(&traffic(650, 50)), true);
    assert_eq!(history.latest().unwrap().down, Some(100.0));
    assert_eq!(history.latest().unwrap().up, Some(0.0));
}

#[test]
fn a_partial_counter_read_cannot_generate_a_recovery_spike() {
    let mut history = TrafficHistory::default();
    history.record(&meta(1, 1000.0), Some(&traffic(1_000_000, 100)), true);
    history.record(&meta(1, 4000.0), Some(&traffic(5, 5)), true);
    history.record(&meta(1, 7000.0), Some(&traffic(1_000_300, 110)), true);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(1, 10000.0), Some(&traffic(1_000_600, 120)), true);
    assert_eq!(history.latest().unwrap().down, Some(100.0));
}

#[test]
fn missing_samples_long_gaps_and_clock_reversal_break_rate_series() {
    let mut history = TrafficHistory::default();
    history.record(&meta(1, 1000.0), Some(&traffic(10, 10)), true);
    history.record(&meta(1, 4000.0), None, true);
    history.record(&meta(1, 7000.0), Some(&traffic(1000, 1000)), true);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(1, 30000.0), Some(&traffic(10000, 10000)), true);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(1, 20000.0), Some(&traffic(11000, 11000)), true);
    assert_eq!(history.points.len(), 1);
    assert_eq!(history.latest().unwrap().down, None);
    history.record(&meta(1, f64::NAN), Some(&traffic(12000, 12000)), true);
    assert_eq!(history.points.len(), 1);
}

#[test]
fn history_is_bounded_and_clears_when_disconnected_or_stats_disabled() {
    let mut history = TrafficHistory::default();
    for index in 0..200 {
        history.record(
            &meta(1, index as f64 * 1000.0),
            Some(&traffic(index, index)),
            true,
        );
    }
    assert_eq!(history.points.len(), 120);
    history.record(&meta(1, 900000.0), Some(&traffic(300, 300)), true);
    assert_eq!(history.points.len(), 1);
    let mut data = meta(1, 901000.0);
    data.stats_enabled = false;
    history.record(&data, Some(&traffic(400, 400)), true);
    assert!(history.points.is_empty());
    data.stats_enabled = true;
    history.record(&data, Some(&traffic(400, 400)), false);
    assert!(history.points.is_empty());
}

#[test]
fn chart_paths_preserve_missing_measurements_and_finite_geometry() {
    let points = vec![
        ChartPoint {
            id: 0,
            x: 0.0,
            y: Some(0.0),
        },
        ChartPoint {
            id: 1,
            x: 1.0,
            y: None,
        },
        ChartPoint {
            id: 0,
            x: 2.0,
            y: Some(100.0),
        },
        ChartPoint {
            id: 0,
            x: 3.0,
            y: Some(f64::NAN),
        },
    ];
    let bounds = PlotBounds {
        width: 480.0,
        height: 196.0,
    };
    let line = paths(&points, 0.0, 3.0, 100.0, false, bounds);
    assert_eq!(line.len(), 2);
    assert!(line.iter().all(|path| !path.contains('L')));
    let area = paths(&points, 0.0, 3.0, 100.0, true, bounds);
    assert!(
        area.iter()
            .all(|path| path.ends_with('Z') && !path.contains("NaN"))
    );
    assert!(paths(&[], 0.0, 0.0, 0.0, true, bounds).is_empty());
    assert_eq!(bytes(0.0), "0 B");
    assert_eq!(bytes(1024.0), "1.0 KiB");
    assert_eq!(bytes(f64::NAN), "—");
}

#[test]
fn config_distribution_counts_each_config_once_and_keeps_failures_separate() {
    let mut all = config(Some(true), None);
    all.test.as_mut().unwrap().real_delay_ok = Some(true);
    all.test.as_mut().unwrap().icmp_ok = Some(true);
    let mut icmp = config(Some(false), None);
    icmp.test.as_mut().unwrap().icmp_ok = Some(true);
    let mut disabled = config(None, None);
    disabled.enabled = false;
    let mut deleted = all.clone();
    deleted.deleted = true;
    let counts = config_distribution(&[
        all,
        config(Some(true), None),
        icmp,
        config(Some(false), None),
        disabled,
        deleted,
    ]);
    assert_eq!(counts, [1, 1, 1, 1, 1]);
    assert_eq!(config_distribution(&[]), [0; 5]);
}

#[test]
fn chart_coordinates_fill_resized_plot_without_scaling_axis_text() {
    for width in [320.0, 960.0] {
        let bounds = PlotBounds {
            width,
            height: 220.0,
        };
        assert_eq!(bounds.x(0.0, 0.0, 100.0), 64.0);
        assert_eq!(bounds.x(100.0, 0.0, 100.0), width - 12.0);
        assert_eq!(bounds.y(100.0, 100.0), 12.0);
        assert_eq!(bounds.y(0.0, 100.0), 194.0);
    }
}

#[test]
fn refresh_restores_server_history_immediately_and_preserves_gaps() {
    use xrat_web::types::TrafficSample;
    let mut data = meta(1, 13_000.0);
    data.traffic_history = vec![
        TrafficSample {
            sampled_at_ms: 1000.0,
            traffic: Some(traffic(100, 10)),
        },
        TrafficSample {
            sampled_at_ms: 4000.0,
            traffic: Some(traffic(1000, 310)),
        },
        TrafficSample {
            sampled_at_ms: 7000.0,
            traffic: None,
        },
        TrafficSample {
            sampled_at_ms: 10000.0,
            traffic: Some(traffic(9000, 9000)),
        },
        TrafficSample {
            sampled_at_ms: 13000.0,
            traffic: Some(traffic(9300, 9300)),
        },
    ];
    let mut history = TrafficHistory::default();
    history.restore(&data, true);
    assert_eq!(history.points.len(), 5);
    assert_eq!(history.points[1].down, Some(300.0));
    assert_eq!(history.points[1].up, Some(100.0));
    assert_eq!(history.points[2].down, None);
    assert_eq!(history.points[3].down, None);
    assert_eq!(history.latest().unwrap().down, Some(100.0));
    let restored = history.clone();
    history.restore(&data, true);
    assert_eq!(history, restored);
    data.stats_enabled = false;
    history.restore(&data, true);
    assert!(history.points.is_empty());
}
