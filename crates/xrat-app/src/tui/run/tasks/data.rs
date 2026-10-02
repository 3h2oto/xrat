use std::sync::Arc;

use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::data::{TuiData, TuiLogs};
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};
use xrat_support::geoip::CachedLookup;

pub fn spawn_enrich_locations(
    db: xrat_db::Database,
    lookup: Arc<CachedLookup>,
    targets: Vec<(xrat_model::ConfigId, String)>,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    if targets.is_empty() {
        return;
    }
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        crate::app::services::dashboard::enrich_locations(db, lookup, targets, |updates| {
            if task_tx
                .send(TuiTaskEvent::LocationsEnriched { updates })
                .is_err()
            {
                tracing::debug!("TUI location enrichment receiver dropped");
            }
        })
        .await;
    });
}

/// The `(config_id, address)` rows still needing a network location lookup after
/// DB test geo and the persistent cache have been applied during load.
pub fn enrichment_targets(data: &TuiData) -> Vec<(xrat_model::ConfigId, String)> {
    data.pending_enrichment.clone()
}

pub fn spawn_reload_data(
    context: AppContext,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::ReloadData;
    if task_tx.send(TuiTaskEvent::Started { kind }).is_err() {
        tracing::debug!(?kind, "TUI task receiver dropped before reload");
    }

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = match TuiData::load(&context, include_deleted).await {
            Ok(data) => TuiTaskEvent::Completed {
                kind,
                message: "reloaded data".to_string(),
                data: Some(data),
            },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: error.to_string(),
                data: None,
            },
        };
        if task_tx.send(event).is_err() {
            tracing::debug!(?kind, "TUI task receiver dropped after reload");
        }
    });
}

pub fn spawn_reload_logs(
    context: AppContext,
    logs_tx: &mpsc::UnboundedSender<crate::app::Result<TuiLogs>>,
) {
    let logs_tx = logs_tx.clone();
    tokio::spawn(async move {
        if logs_tx.send(TuiLogs::load(&context).await).is_err() {
            tracing::debug!("TUI logs receiver dropped after reload");
        }
    });
}

/// Probe engine versions off the critical path so the first frame paints
/// without waiting up to two seconds per engine.
pub fn spawn_probe_engines(
    context: AppContext,
    engines_tx: &mpsc::UnboundedSender<Vec<crate::tui::data::EngineInfo>>,
) {
    let engines_tx = engines_tx.clone();
    tokio::spawn(async move {
        if engines_tx
            .send(
                crate::app::services::engine_probe::probe_engines(
                    &context,
                    &crate::app::services::engine_probe::ProcessRuntimeEngineProbe,
                )
                .await,
            )
            .is_err()
        {
            tracing::debug!("TUI engine probe receiver dropped");
        }
    });
}

/// Clear all persisted events from the database, then refresh the logs card off
/// the event loop. This is the destructive DB clear; engine log files are left
/// untouched.
pub async fn run_clear_events(
    context: &AppContext,
    app: &mut TuiApp,
    logs_tx: &mpsc::UnboundedSender<crate::app::Result<TuiLogs>>,
) {
    match crate::app::services::dashboard::DashboardService::new(context)
        .clear_events()
        .await
    {
        Ok(count) => {
            spawn_reload_logs(context.clone(), logs_tx);
            app.push_log(format!("OK  cleared {count} persisted event(s) from db"));
        }
        Err(error) => app.push_log(format!("ERR clear events failed: {error}")),
    }
}
