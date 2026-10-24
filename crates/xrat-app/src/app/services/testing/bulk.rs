use super::bulk_progress::{
    bulk_progress_bar, finish_bulk_progress, spawn_config_test, update_bulk_progress,
};
use super::*;
use xrat_support::cancel::CancellationReceiver;

pub(crate) struct BulkProgress {
    pub updates: tokio::sync::mpsc::UnboundedSender<TestProgressUpdate>,
    pub started: Option<tokio::sync::oneshot::Sender<i64>>,
}

pub(crate) struct BulkTestOutcome {
    pub run_id: i64,
    pub rows: Vec<TestOutputRow>,
}

pub(crate) async fn run_bulk_for_configs(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
) -> crate::app::Result<Vec<TestOutputRow>> {
    run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        run_kind,
        show_progress,
        None,
        None,
    )
    .await
    .map(|outcome| outcome.rows)
}

pub(crate) async fn run_bulk_for_configs_cancellable(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
    cancel_rx: Option<CancellationReceiver>,
    mut progress_tx: Option<BulkProgress>,
) -> crate::app::Result<BulkTestOutcome> {
    let run_id = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: run_kind.to_string(),
        })
        .await?;
    if let Some(progress) = &mut progress_tx
        && let Some(started) = progress.started.take()
    {
        let _ = started.send(run_id);
    }

    let total = configs.len();
    for config in &configs {
        super::activity::queued(&settings, config.id);
    }
    let concurrency = resolve_concurrency(settings.concurrency)?;
    let progress = bulk_progress_bar(total, show_progress);
    if run_kind == "rotation" && total > 0 {
        crate::app::events::record(
            &context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_ROTATION,
            "rotation_bulk_progress",
            "rotation_bulk_started".to_string(),
            None,
            None,
            Some(format!("{{\"done\":0,\"total\":{total}}}")),
        )
        .await;
    }
    let mut next_config = configs.into_iter();
    let mut join_set = JoinSet::new();
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut outputs = Vec::with_capacity(total);

    let cancelled = || cancel_rx.as_ref().is_some_and(|rx| rx.is_cancelled());

    for _ in 0..concurrency {
        if cancelled() {
            break;
        }
        let Some(config) = next_config.next() else {
            break;
        };
        spawn_config_test(
            &mut join_set,
            context.db.clone(),
            config,
            settings.clone(),
            Some(run_id),
        );
    }

    let cancel_future = async {
        if let Some(rx) = &cancel_rx {
            rx.cancelled().await;
        } else {
            std::future::pending::<()>().await;
        }
    };
    tokio::pin!(cancel_future);

    loop {
        if cancelled() {
            join_set.abort_all();
            break;
        }

        if join_set.is_empty() {
            break;
        }

        let joined = tokio::select! {
            _ = &mut cancel_future => {
                join_set.abort_all();
                break;
            }
            res = join_set.join_next() => {
                match res {
                    Some(res) => res,
                    None => break,
                }
            }
        };

        match joined {
            Ok(Ok(output)) => {
                completed += 1;
                if output.status != TestStatus::Ok {
                    failed += 1;
                }
                let config_id = output.id;
                outputs.push(output);
                update_bulk_progress(&progress, completed, failed);
                if let Some(tx) = &progress_tx {
                    let _ = tx.updates.send(TestProgressUpdate {
                        config_id,
                        done: completed,
                        total,
                    });
                }
            }
            Ok(Err(error)) => {
                stop_workers(&mut join_set).await;
                return Err(error);
            }
            Err(join_error) if join_error.is_cancelled() => {}
            Err(join_error) => {
                stop_workers(&mut join_set).await;
                return Err(join_error.into());
            }
        }

        if cancelled() {
            join_set.abort_all();
            break;
        }

        if let Some(config) = next_config.next() {
            spawn_config_test(
                &mut join_set,
                context.db.clone(),
                config,
                settings.clone(),
                Some(run_id),
            );
        }
    }
    stop_workers(&mut join_set).await;
    finish_bulk_progress(progress, completed, failed);
    if run_kind == "rotation" {
        crate::app::events::record(
            &context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_ROTATION,
            "rotation_bulk_progress",
            "rotation_bulk_finished".to_string(),
            None,
            None,
            Some(format!("{{\"done\":{completed},\"total\":{total}}}")),
        )
        .await;
    }

    let level = if failed > 0 && failed == completed {
        crate::app::events::LEVEL_WARN
    } else {
        crate::app::events::LEVEL_INFO
    };
    crate::app::events::record(
        &context.db,
        level,
        crate::app::events::SOURCE_TEST,
        "test_run",
        format!(
            "Tested {completed} config(s): {} passed, {failed} failed",
            completed.saturating_sub(failed)
        ),
        None,
        None,
        Some(format!("{{\"run_id\":{run_id},\"kind\":\"{run_kind}\"}}")),
    )
    .await;

    Ok(BulkTestOutcome {
        run_id,
        rows: outputs,
    })
}

async fn stop_workers(workers: &mut JoinSet<crate::app::Result<TestOutputRow>>) {
    workers.abort_all();
    while workers.join_next().await.is_some() {}
}

#[cfg(test)]
mod tests;
