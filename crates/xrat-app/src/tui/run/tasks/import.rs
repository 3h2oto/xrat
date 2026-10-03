use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};
use xrat_db::ImportSource;
use xrat_model::Node;

pub enum TuiImport {
    Config {
        source: ImportSource,
        node: Box<Node>,
    },
    Subscription {
        url: String,
        name: String,
    },
}

pub fn spawn_import(
    context: AppContext,
    import: TuiImport,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::Import;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = match run_import(&context, import).await {
            Ok(message) => match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message,
                    data: Some(data),
                },
                Err(error) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("import completed but reload failed: {error}"),
                    data: None,
                },
            },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: format!("import failed: {error}"),
                data: None,
            },
        };
        let _ = task_tx.send(event);
    });
}

async fn run_import(context: &AppContext, import: TuiImport) -> crate::app::Result<String> {
    match import {
        TuiImport::Config { source, node } => {
            context
                .db
                .import_nodes(&source, std::slice::from_ref(node.as_ref()))
                .await?;
            Ok("added 1 config".to_string())
        }
        TuiImport::Subscription { url, name } => {
            let (source, nodes) = crate::app::import::load_nodes_async(&url).await?;
            let summary =
                crate::app::import::persist_nodes(&context.db, source, &nodes, Some(&name)).await?;
            Ok(format!(
                "imported {} configs into {name}",
                summary.imported_configs
            ))
        }
    }
}

#[cfg(test)]
mod tests;
