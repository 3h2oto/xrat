use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::app::services::ToggleOutcome;
use crate::tui::app::{TuiApp, TuiConfigCommand};
use crate::tui::task::TuiTaskEvent;

/// Apply a single-config command. Enable/disable mutate the affected row in
/// place; commands that change which rows are visible (delete/restore/purge)
/// spawn a background reload instead of blocking the event loop on a full load.
pub async fn run_config_command(
    context: &AppContext,
    app: &mut TuiApp,
    command: TuiConfigCommand,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    match command {
        TuiConfigCommand::Enable(id) => apply_enabled(context, app, id, true).await,
        TuiConfigCommand::Disable(id) => apply_enabled(context, app, id, false).await,
        TuiConfigCommand::Restore(id) => {
            apply_reload(
                app,
                context
                    .services()
                    .lifecycle
                    .restore(&id.to_string())
                    .await
                    .map(|_| ()),
                id,
                "restored",
                task_tx,
                context,
            );
        }
        TuiConfigCommand::SoftDelete(id) => {
            apply_reload(
                app,
                context
                    .services()
                    .lifecycle
                    .delete(&id.to_string(), false)
                    .await
                    .map(|_| ()),
                id,
                "soft deleted",
                task_tx,
                context,
            );
        }
        TuiConfigCommand::Purge(id) => {
            apply_reload(
                app,
                context
                    .services()
                    .lifecycle
                    .delete(&id.to_string(), true)
                    .await
                    .map(|_| ()),
                id,
                "purged",
                task_tx,
                context,
            );
        }
    }
}

async fn apply_enabled(context: &AppContext, app: &mut TuiApp, id: i64, enabled: bool) {
    let lifecycle = context.services().lifecycle;
    let result = if enabled {
        lifecycle.enable(&id.to_string()).await
    } else {
        lifecycle.disable(&id.to_string()).await
    };
    match result {
        Ok(ToggleOutcome::DeletedConfig) => {
            app.push_log(format!("Config {id} is deleted; restore it first"))
        }
        Ok(_) => {
            app.data.set_config_enabled(id, enabled);
            let verb = if enabled { "enabled" } else { "disabled" };
            app.push_log(format!("OK  {verb} config {id}"));
        }
        Err(error) => app.push_log(format!("ERR operation failed: {error}")),
    }
}

fn apply_reload(
    app: &mut TuiApp,
    result: crate::app::Result<()>,
    id: i64,
    verb: &str,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
    context: &AppContext,
) {
    match result {
        Ok(_) => {
            super::spawn_reload_data(context.clone(), app.config_list.include_deleted, task_tx);
            app.push_log(format!("OK  {verb} config {id}"));
        }
        Err(error) => app.push_log(format!("ERR operation failed: {error}")),
    }
}
