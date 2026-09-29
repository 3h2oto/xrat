use crate::app::context::AppContext;
use crate::app::services::testing::TestRunRequest;

use super::bulk;
use super::settings::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TestProgressUpdate {
    pub(crate) config_id: i64,
    pub(crate) done: usize,
    pub(crate) total: usize,
}

#[allow(dead_code)]
pub(crate) async fn run_bulk_for_config_ids_cancellable(
    request: &TestRunRequest,
    context: &AppContext,
    config_ids: &[i64],
    cancel_rx: crate::support::cancel::CancellationReceiver,
) -> crate::app::Result<usize> {
    let settings = resolve_test_settings(request, &context.app_config, &context.runtime_paths)?;
    let mut configs = Vec::with_capacity(config_ids.len());

    for config_id in config_ids {
        if let Some(config) = context.db.get_config_by_id(*config_id).await? {
            configs.push(config);
        }
    }

    if configs.is_empty() {
        return Ok(0);
    }

    let rows = bulk::run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        "tui",
        false,
        Some(cancel_rx),
    )
    .await?;
    Ok(rows.len())
}

pub(crate) async fn run_bulk_for_config_ids_with_progress(
    request: &TestRunRequest,
    context: &AppContext,
    config_ids: &[i64],
    cancel_rx: crate::support::cancel::CancellationReceiver,
    progress_tx: tokio::sync::mpsc::UnboundedSender<TestProgressUpdate>,
) -> crate::app::Result<usize> {
    let settings = resolve_test_settings(request, &context.app_config, &context.runtime_paths)?;
    let mut configs = Vec::with_capacity(config_ids.len());

    for config_id in config_ids {
        if let Some(config) = context.db.get_config_by_id(*config_id).await? {
            configs.push(config);
        }
    }

    if configs.is_empty() {
        return Ok(0);
    }

    let rows = bulk::run_bulk_for_configs_with_progress(
        context,
        settings,
        configs,
        "tui",
        Some(cancel_rx),
        progress_tx,
    )
    .await?;
    Ok(rows.len())
}
