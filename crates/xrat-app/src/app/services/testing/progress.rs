use crate::app::context::AppContext;
use xrat_model::ConfigId;

use super::{TestRunRequest, resolve_test_settings, run_bulk_for_configs_cancellable};

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

    let rows = run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        "tui",
        false,
        Some(cancel_rx),
        Some(progress_tx),
    )
    .await?;
    Ok(rows.len())
}
