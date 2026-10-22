use std::collections::BTreeMap;
use xrat_model::ConfigId;
use xrat_web::types::{ConfigTestProgress, TestMetric, TestMetricProgress, TestMetricState};

#[derive(Default)]
pub(crate) struct TestActivityProgress {
    pub done: usize,
    pub total: usize,
    pub rows: BTreeMap<i64, ConfigTestProgress>,
}

impl TestActivityProgress {
    pub fn apply(&mut self, activity: super::TestActivity) {
        use super::TestActivity;
        match activity {
            #[cfg(any(windows, test))]
            TestActivity::Snapshot { row } => {
                self.rows.insert(row.config_id, row);
            }
            TestActivity::Queued { config_id, metrics } => {
                self.rows.insert(
                    config_id.0,
                    ConfigTestProgress {
                        config_id: config_id.0,
                        metrics: metrics
                            .into_iter()
                            .map(|metric| TestMetricProgress {
                                metric: metric_type(metric),
                                state: TestMetricState::Queued,
                            })
                            .collect(),
                    },
                );
            }
            TestActivity::Started { config_id, metric } => {
                self.metric(config_id, metric, TestMetricState::Running)
            }
            TestActivity::Measured {
                config_id,
                metric,
                passed,
            } => self.metric(
                config_id,
                metric,
                if passed {
                    TestMetricState::Passed
                } else {
                    TestMetricState::Failed
                },
            ),
            TestActivity::Finished { config_id, failed } => {
                if let Some(row) = self.rows.get_mut(&config_id.0) {
                    for metric in &mut row.metrics {
                        metric.state = match metric.state {
                            TestMetricState::Running if failed => TestMetricState::Failed,
                            TestMetricState::Queued | TestMetricState::Running => {
                                TestMetricState::Skipped
                            }
                            other => other,
                        };
                    }
                }
            }
        }
    }

    fn metric(&mut self, id: ConfigId, metric: super::TestMetric, state: TestMetricState) {
        if let Some(row) = self.rows.get_mut(&id.0)
            && let Some(progress) = row
                .metrics
                .iter_mut()
                .find(|progress| progress.metric == metric_type(metric))
        {
            progress.state = state;
        }
    }
}

fn metric_type(metric: super::TestMetric) -> TestMetric {
    match metric {
        super::TestMetric::Icmp => TestMetric::Icmp,
        super::TestMetric::Tcp => TestMetric::Tcp,
        super::TestMetric::RealDelay => TestMetric::RealDelay,
        super::TestMetric::Download => TestMetric::Download,
        super::TestMetric::Upload => TestMetric::Upload,
    }
}
