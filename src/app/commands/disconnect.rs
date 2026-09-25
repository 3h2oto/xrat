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
    use crate::app::config::AppConfig;
    use crate::app::context::RuntimePaths;
    use crate::db::{Database, DatabaseConnectionConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

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
        let root = std::env::temp_dir().join(format!(
            "xrat-command-{prefix}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root should be created");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: root.join("db.sqlite"),
        };
        let db = Database::connect(&database_config)
            .await
            .expect("database should connect");
        AppContext {
            db,
            app_config: AppConfig::default(),
            runtime_paths: RuntimePaths {
                root_dir: root.clone(),
                database_config,
                database_path: root.join("db.sqlite"),
                database_label: root.join("db.sqlite").display().to_string(),
                config_path: root.join("config.toml"),
                runtime_dir: root.join("runtime"),
                xray_path: "xray".into(),
                v2ray_path: "v2ray".into(),
                sing_box_path: "sing-box".into(),
            },
        }
    }
}
