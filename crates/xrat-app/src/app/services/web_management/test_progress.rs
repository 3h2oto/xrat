use super::*;
use std::collections::BTreeMap;
use xrat_web::types::{
    ConfigTestProgress, TestJobProgress, TestMetric, TestMetricProgress, TestMetricState,
};

#[derive(Default)]
struct Progress {
    done: usize,
    total: usize,
    rows: BTreeMap<i64, ConfigTestProgress>,
}

impl Progress {
    fn apply(&mut self, activity: testing::TestActivity) {
        use testing::TestActivity;
        match activity {
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

    fn metric(&mut self, id: ConfigId, metric: testing::TestMetric, state: TestMetricState) {
        if let Some(row) = self.rows.get_mut(&id.0)
            && let Some(progress) = row
                .metrics
                .iter_mut()
                .find(|progress| progress.metric == metric_type(metric))
        {
            progress.state = state;
        }
    }

    async fn save(&self, db: &xrat_db::Database, job_id: &str) {
        let progress = TestJobProgress {
            message: format!("Tested {} of {} configs", self.done, self.total),
            tests: self.rows.values().cloned().collect(),
        };
        if let Ok(payload) = serde_json::to_string(&progress)
            && let Err(error) = db.web_job_progress(job_id, &payload, timestamp()).await
        {
            tracing::warn!(%error, "could not record test activity");
        }
    }
}

fn metric_type(metric: testing::TestMetric) -> TestMetric {
    match metric {
        testing::TestMetric::Icmp => TestMetric::Icmp,
        testing::TestMetric::Tcp => TestMetric::Tcp,
        testing::TestMetric::RealDelay => TestMetric::RealDelay,
        testing::TestMetric::Download => TestMetric::Download,
        testing::TestMetric::Upload => TestMetric::Upload,
    }
}

pub(super) async fn run(
    context: &app::context::AppContext,
    ids: &[ConfigId],
    flag: &xrat_support::cancel::CancellationFlag,
    job_id: &str,
) -> app::Result<String> {
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
    let (activity_tx, mut activity_rx) = tokio::sync::mpsc::unbounded_channel();
    let db = context.db.clone();
    let progress_id = job_id.to_string();
    let total = ids.len();
    let drain = tokio::spawn(async move {
        let mut progress = Progress {
            total,
            ..Default::default()
        };
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(200));
        loop {
            tokio::select! {
                Some(update) = progress_rx.recv() => {
                    let update: testing::TestProgressUpdate = update;
                    progress.done = update.done;
                    progress.total = update.total;
                }
                Some(update) = activity_rx.recv() => progress.apply(update),
                _ = interval.tick(), if !progress_rx.is_closed() || !activity_rx.is_closed() => progress.save(&db, &progress_id).await,
                else => break,
            }
        }
        progress.save(&db, &progress_id).await;
    });
    let result = testing::run_bulk_for_config_ids_with_activity(
        &testing::TestRunRequest::default(),
        context,
        ids,
        xrat_support::cancel::CancellationReceiver::from_flag(flag),
        progress_tx,
        activity_tx,
    )
    .await;
    let _ = drain.await;
    result.map(|count| format!("Tested {count} configs"))
}

#[cfg(test)]
mod tests;
