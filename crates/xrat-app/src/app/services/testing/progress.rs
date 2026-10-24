use crate::app::context::AppContext;
use xrat_model::ConfigId;

use super::bulk::BulkProgress;
use super::{TestRunRequest, resolve_test_settings, run_bulk_for_configs_cancellable};

pub(crate) struct TestRunOutcome {
    pub run_id: Option<i64>,
    pub completed: usize,
    pub total: usize,
    #[cfg_attr(not(any(windows, test)), allow(dead_code))]
    pub job_id: Option<String>,
    pub local_results: Option<Vec<super::TestOutputRow>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TestProgressUpdate {
    pub config_id: ConfigId,
    pub done: usize,
    pub total: usize,
}

pub async fn run_bulk_for_config_ids_with_progress(
    request: &TestRunRequest,
    context: &AppContext,
    config_ids: &[ConfigId],
    cancel_rx: xrat_support::cancel::CancellationReceiver,
    progress_tx: tokio::sync::mpsc::UnboundedSender<TestProgressUpdate>,
) -> crate::app::Result<usize> {
    run_with_activity(
        request,
        context,
        config_ids,
        cancel_rx,
        BulkProgress {
            updates: progress_tx,
            started: None,
        },
        None,
        "tui",
    )
    .await
    .map(|outcome| outcome.completed)
}

pub(crate) async fn run_bulk_for_config_ids_with_activity(
    request: &TestRunRequest,
    context: &AppContext,
    config_ids: &[ConfigId],
    cancel_rx: xrat_support::cancel::CancellationReceiver,
    progress_tx: tokio::sync::mpsc::UnboundedSender<TestProgressUpdate>,
    activity_tx: super::TestActivitySender,
) -> crate::app::Result<usize> {
    run_with_activity(
        request,
        context,
        config_ids,
        cancel_rx,
        BulkProgress {
            updates: progress_tx,
            started: None,
        },
        Some(activity_tx),
        "web",
    )
    .await
    .map(|outcome| outcome.completed)
}

pub(crate) async fn run_machine_probe(
    request: &TestRunRequest,
    context: &AppContext,
    config_ids: &[ConfigId],
    cancel_rx: xrat_support::cancel::CancellationReceiver,
    progress_tx: tokio::sync::mpsc::UnboundedSender<TestProgressUpdate>,
    started: tokio::sync::oneshot::Sender<i64>,
    activity: super::TestActivitySender,
) -> crate::app::Result<TestRunOutcome> {
    run_with_activity(
        request,
        context,
        config_ids,
        cancel_rx,
        BulkProgress {
            updates: progress_tx,
            started: Some(started),
        },
        Some(activity),
        "machine",
    )
    .await
}

async fn run_with_activity(
    request: &TestRunRequest,
    context: &AppContext,
    config_ids: &[ConfigId],
    cancel_rx: xrat_support::cancel::CancellationReceiver,
    progress_tx: BulkProgress,
    activity: Option<super::TestActivitySender>,
    run_kind: &str,
) -> crate::app::Result<TestRunOutcome> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        return super::machine::run(
            request,
            client,
            config_ids,
            cancel_rx,
            progress_tx,
            activity,
        )
        .await;
    }
    if cancel_rx.is_cancelled() {
        return Ok(TestRunOutcome {
            job_id: None,
            local_results: Some(Vec::new()),
            run_id: None,
            completed: 0,
            total: 0,
        });
    }
    let mut settings = resolve_test_settings(request, &context.app_config, &context.runtime_paths)?;
    settings.activity = activity;
    let mut configs = Vec::with_capacity(config_ids.len());

    for config_id in config_ids {
        if cancel_rx.is_cancelled() {
            return Ok(TestRunOutcome {
                job_id: None,
                local_results: Some(Vec::new()),
                run_id: None,
                completed: 0,
                total: configs.len(),
            });
        }
        if let Some(config) = context.db.get_config_by_id(*config_id).await? {
            configs.push(config);
        }
    }

    if configs.is_empty() {
        return Ok(TestRunOutcome {
            job_id: None,
            local_results: Some(Vec::new()),
            run_id: None,
            completed: 0,
            total: 0,
        });
    }

    let total = configs.len();
    let outcome = run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        run_kind,
        false,
        Some(cancel_rx),
        Some(progress_tx),
    )
    .await?;
    let completed = outcome.rows.len();
    Ok(TestRunOutcome {
        job_id: None,
        local_results: Some(outcome.rows),
        run_id: Some(outcome.run_id),
        completed,
        total,
    })
}
