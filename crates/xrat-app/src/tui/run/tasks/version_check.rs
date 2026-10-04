use crate::app::services::releases::ReleaseService;
use tokio::sync::mpsc;

pub fn spawn_version_check(tx: mpsc::UnboundedSender<String>) {
    tokio::spawn(async move {
        match ReleaseService::default()
            .newer_tag(env!("CARGO_PKG_VERSION"), 8)
            .await
        {
            Ok(Some(tag)) => {
                if tx.send(tag).is_err() {
                    tracing::debug!(
                        operation = "release_check_send",
                        "TUI version receiver dropped"
                    );
                }
            }
            Ok(None) => {}
            Err(error) => {
                tracing::debug!(operation = "release_check", %error, "optional release check failed")
            }
        }
    });
}
