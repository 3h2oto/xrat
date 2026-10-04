use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::{BulkKind, BulkOp, TuiApp};
use crate::tui::task::TuiTaskEvent;

pub async fn run_bulk_op(
    context: &AppContext,
    app: &mut TuiApp,
    op: BulkOp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let ids = app.bulk_config_ids(op);
    if ids.is_empty() {
        return;
    }

    let lifecycle = context.services().lifecycle;
    let result = match op.kind() {
        BulkKind::SoftDelete => lifecycle.delete_many(&ids, false).await,
        BulkKind::Purge => lifecycle.delete_many(&ids, true).await,
        BulkKind::Restore => lifecycle.restore_many(&ids).await,
    };

    match result {
        Ok(affected) => {
            super::spawn_reload_data(context.clone(), app.config_list.include_deleted, task_tx);
            let message = format!("{} {affected} {} configs", op.verb(), op.target());
            app.push_log(format!("OK  {message}"));
        }
        Err(error) => {
            let msg = format!("operation failed: {error}");
            app.push_log(format!("ERR {msg}"));
        }
    }
}
