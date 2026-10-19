use super::*;

#[test]
fn stage_progress_preserves_finished_results_and_skips_unstarted_metrics() {
    let mut progress = Progress::default();
    let id = ConfigId(7);
    progress.apply(testing::TestActivity::Queued {
        config_id: id,
        metrics: vec![
            testing::TestMetric::Tcp,
            testing::TestMetric::RealDelay,
            testing::TestMetric::Download,
        ],
    });
    progress.apply(testing::TestActivity::Started {
        config_id: id,
        metric: testing::TestMetric::Tcp,
    });
    assert_eq!(progress.rows[&7].metrics[0].state, TestMetricState::Running);
    progress.apply(testing::TestActivity::Measured {
        config_id: id,
        metric: testing::TestMetric::Tcp,
        passed: false,
    });
    progress.apply(testing::TestActivity::Finished {
        config_id: id,
        failed: false,
    });
    assert_eq!(
        progress.rows[&7]
            .metrics
            .iter()
            .map(|metric| metric.state)
            .collect::<Vec<_>>(),
        vec![
            TestMetricState::Failed,
            TestMetricState::Skipped,
            TestMetricState::Skipped
        ]
    );
}

#[test]
fn runner_errors_fail_the_current_metric_without_changing_other_rows() {
    let mut progress = Progress::default();
    for id in [1, 2] {
        progress.apply(testing::TestActivity::Queued {
            config_id: ConfigId(id),
            metrics: vec![testing::TestMetric::Tcp],
        });
    }
    progress.apply(testing::TestActivity::Started {
        config_id: ConfigId(1),
        metric: testing::TestMetric::Tcp,
    });
    progress.apply(testing::TestActivity::Finished {
        config_id: ConfigId(1),
        failed: true,
    });
    assert_eq!(progress.rows[&1].metrics[0].state, TestMetricState::Failed);
    assert_eq!(progress.rows[&2].metrics[0].state, TestMetricState::Queued);
}
