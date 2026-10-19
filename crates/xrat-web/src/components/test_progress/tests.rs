use super::*;
use xrat_web::types::TestMetricProgress;

#[test]
fn test_columns_follow_their_metric_and_other_cells_are_unchanged() {
    let progress = vec![ConfigTestProgress {
        config_id: 7,
        metrics: vec![
            TestMetricProgress {
                metric: TestMetric::Tcp,
                state: TestMetricState::Running,
            },
            TestMetricProgress {
                metric: TestMetric::RealDelay,
                state: TestMetricState::Queued,
            },
        ],
    }];
    assert_eq!(
        column_state(&progress, 7, "tcp"),
        Some(TestMetricState::Running)
    );
    assert_eq!(
        column_state(&progress, 7, "delay"),
        Some(TestMetricState::Queued)
    );
    assert_eq!(
        column_state(&progress, 7, "test"),
        Some(TestMetricState::Running)
    );
    assert_eq!(column_state(&progress, 7, "protocol"), None);
    assert_eq!(column_state(&progress, 7, "icmp"), None);
    assert_eq!(column_state(&progress, 8, "tcp"), None);
}

#[test]
fn finished_rows_return_to_their_measured_cells() {
    let progress = vec![ConfigTestProgress {
        config_id: 7,
        metrics: vec![
            TestMetricProgress {
                metric: TestMetric::Tcp,
                state: TestMetricState::Failed,
            },
            TestMetricProgress {
                metric: TestMetric::RealDelay,
                state: TestMetricState::Skipped,
            },
        ],
    }];
    assert_eq!(column_state(&progress, 7, "tcp"), None);
    assert_eq!(column_state(&progress, 7, "test"), None);
}
