use super::*;
use xrat_web::types::TestJobProgress;
#[cfg(test)]
use xrat_web::types::TestMetricState;

type Progress = testing::TestActivityProgress;

async fn save(progress: &Progress, db: &xrat_db::Database, job_id: &str) {
    let payload = TestJobProgress {
        message: format!("Tested {} of {} configs", progress.done, progress.total),
        tests: progress.rows.values().cloned().collect(),
    };
    if let Ok(payload) = serde_json::to_string(&payload)
        && let Err(error) = db.web_job_progress(job_id, &payload, timestamp()).await
    {
        tracing::warn!(%error, "could not record test activity");
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
                _ = interval.tick(), if !progress_rx.is_closed() || !activity_rx.is_closed() => save(&progress, &db, &progress_id).await,
                else => break,
            }
        }
        save(&progress, &db, &progress_id).await;
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
