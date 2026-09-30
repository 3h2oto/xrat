use tokio::sync::mpsc;
use tracing::Instrument;

use crate::app::context::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_test_batch(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    if app.task_state.running.is_some() {
        return;
    }

    let config_ids = app.test_config_ids();
    if config_ids.is_empty() {
        return;
    }
    app.data.clear_test_fields_for_configs(&config_ids);
    app.testing_config_ids = config_ids.clone();

    let kind = TuiTaskKind::TestBatch;
    let request = test_run_request_for_app(app);
    let include_deleted = app.config_list.include_deleted;
    let (token, receiver) = app.task_state.start(kind);
    if task_tx.send(TuiTaskEvent::Started { kind }).is_err() {
        tracing::debug!(?kind, "TUI task receiver dropped before batch test");
    }

    let (progress_tx, mut progress_rx) =
        mpsc::unbounded_channel::<crate::app::services::testing::TestProgressUpdate>();
    let task_tx_clone = task_tx.clone();
    let progress_context = context.clone();
    tokio::spawn(async move {
        while let Some(update) = progress_rx.recv().await {
            match progress_context
                .services()
                .configs
                .detail(update.config_id)
                .await
            {
                Ok(Some(row)) => {
                    if task_tx_clone
                        .send(TuiTaskEvent::ConfigTested {
                            row: row.into(),
                            done: update.done,
                            total: update.total,
                        })
                        .is_err()
                    {
                        tracing::debug!(?kind, "TUI progress receiver dropped during batch test");
                    }
                }
                Err(error) => {
                    tracing::debug!(config_id = update.config_id, %error, "failed to load tested config detail");
                    if task_tx_clone
                        .send(TuiTaskEvent::Progress {
                            kind,
                            done: update.done,
                            total: update.total,
                        })
                        .is_err()
                    {
                        tracing::debug!(?kind, "TUI progress receiver dropped during batch test");
                    }
                }
                Ok(None) => {
                    tracing::debug!(config_id = update.config_id, "tested config detail missing");
                    if task_tx_clone
                        .send(TuiTaskEvent::Progress {
                            kind,
                            done: update.done,
                            total: update.total,
                        })
                        .is_err()
                    {
                        tracing::debug!(?kind, "TUI progress receiver dropped during batch test");
                    }
                }
            }
        }
    });

    let task_tx = task_tx.clone();
    let batch_span = tracing::debug_span!("tui_test_batch", config_count = config_ids.len());
    tokio::spawn(
        async move {
            let result = crate::app::services::testing::run_bulk_for_config_ids_with_progress(
                &request,
                &context,
                &config_ids,
                receiver,
                progress_tx,
            )
            .await;

            let was_cancelled = token.is_cancelled();
            let event = match result {
                Ok(_) if was_cancelled => TuiTaskEvent::Cancelled { kind },
                Ok(tested) => match TuiData::load(&context, include_deleted).await {
                    Ok(data) => TuiTaskEvent::Completed {
                        kind,
                        message: format!("tested {tested} configs"),
                        data: Some(data),
                    },
                    Err(error) => TuiTaskEvent::Failed {
                        kind,
                        error: format!("test completed but reload failed: {error}"),
                        data: None,
                    },
                },
                Err(_) if was_cancelled => TuiTaskEvent::Cancelled { kind },
                Err(error) => TuiTaskEvent::Failed {
                    kind,
                    error: error.to_string(),
                    data: None,
                },
            };
            if task_tx.send(event).is_err() {
                tracing::debug!(?kind, "TUI task receiver dropped after batch test");
            }
        }
        .instrument(batch_span),
    );
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn test_run_request_for_app(
    app: &TuiApp,
) -> crate::app::services::testing::TestRunRequest {
    let stage_enabled = |name: &str| app.data.test_stage_names.iter().any(|stage| stage == name);

    crate::app::services::testing::TestRunRequest {
        enabled_only: false,
        active_only: false,
        skip_icmp: !stage_enabled("icmp"),
        skip_tcp: true,
        skip_real_delay: !stage_enabled("real_delay"),
        skip_download: !stage_enabled("download"),
        skip_upload: true,
        test_url: None,
        download_url: None,
        upload_url: None,
        icmp_timeout_ms: None,
        tcp_timeout_ms: None,
        real_delay_timeout_ms: None,
        download_timeout_ms: None,
        upload_timeout_ms: None,
        concurrency: Some(app.test_state.concurrency as i32),
    }
}
