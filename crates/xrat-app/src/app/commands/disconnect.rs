use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::context::AppContext;
use crate::app::services::runtime_control;
use crate::cli::DisconnectArgs;

pub async fn run(context: &AppContext, args: &DisconnectArgs) -> crate::app::Result<()> {
    let control = runtime_control::daemon_control(context);
    let progress = CliProgress::spinner(!args.json, "disconnecting runtime");
    let result = control.disconnect().await;
    progress.finish_and_clear();
    let stopped_session = result?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "stopped_session": stopped_session,
                "message": if stopped_session {
                    "Disconnected active runtime session"
                } else {
                    "No active runtime session"
                },
            }))?
        );
        return Ok(());
    }

    if stopped_session {
        println!(
            "{}",
            output::success(
                "Disconnected active runtime session.",
                output::color_enabled()
            )
        );
    } else {
        println!(
            "{}",
            output::notice("No active runtime session.", output::color_enabled())
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppError;

    #[tokio::test]
    async fn disconnect_returns_daemon_unreachable_hint() {
        let context = test_context("disconnect-daemon-hint").await;
        let err = run(&context, &DisconnectArgs { json: false })
            .await
            .expect_err("disconnect should require daemon reachability");
        match err {
            AppError::InvalidArgument(message) => {
                assert!(message.contains("xrat daemon start"));
                assert!(message.contains("daemon is not running"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    async fn test_context(prefix: &str) -> AppContext {
        crate::app::tests::TestAppBuilder::new(prefix).build().await
    }
}
