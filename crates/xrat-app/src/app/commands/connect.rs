use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::context::AppContext;
use crate::app::services::runtime_control;
use crate::cli::ConnectArgs;

pub async fn run(context: &AppContext, args: &ConnectArgs) -> crate::app::Result<()> {
    let services = context.services();
    let config_id = services
        .lifecycle
        .resolve_config_id(&xrat_model::ConfigRef::from(args.id.as_str()))
        .await?;
    let control = runtime_control::daemon_control(context);
    let progress = CliProgress::spinner(!args.json, format!("connecting config {config_id}"));
    let result = control.connect(config_id).await;
    progress.finish_and_clear();
    let outcome = result?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "connected",
                "daemon": true,
                "config": { "id": outcome.config_id },
                "session": { "id": outcome.session_id, "pid": outcome.pid },
            }))?
        );
    } else {
        println!(
            "{}",
            output::success(
                format!("Connected config {} via daemon.", outcome.config_id),
                output::color_enabled()
            )
        );
        println!(
            "{}",
            output::format_kv(
                None,
                &[
                    ("session", outcome.session_id.to_string()),
                    ("pid", outcome.pid.to_string()),
                ],
                output::color_enabled(),
            )
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppError;

    #[tokio::test]
    async fn connect_returns_daemon_unreachable_hint() {
        let context = test_context("connect-daemon-hint").await;
        seed_config(&context).await;
        let err = run(
            &context,
            &ConnectArgs {
                id: "1".to_string(),
                json: false,
            },
        )
        .await
        .expect_err("connect should require daemon reachability");
        match err {
            AppError::InvalidArgument(message) => {
                assert!(message.contains("xrat daemon start"));
                assert!(message.contains("daemon is not running"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    async fn seed_config(context: &AppContext) {
        let source = xrat_db::ImportSource {
            kind: xrat_db::SourceKind::File,
            value: "seed.txt".to_string(),
            name: None,
        };
        context
            .db
            .import_nodes(
                &source,
                &[xrat_model::Node {
                    protocol: xrat_model::Protocol::Vless,
                    address: "example.com".to_string(),
                    port: 443,
                    username: None,
                    uuid: Some("uuid-123".to_string()),
                    password: None,
                    method: None,
                    network: "ws".to_string(),
                    tls: Some("tls".to_string()),
                    sni: Some("cdn.example.com".to_string()),
                    host: Some("cdn.example.com".to_string()),
                    path: Some("/socket".to_string()),
                    name: Some("seed".to_string()),
                    extensions: None,
                    raw_config: "vless://uuid-123@example.com:443?type=ws#seed".to_string(),
                }],
            )
            .await
            .expect("import should succeed");
    }

    async fn test_context(prefix: &str) -> AppContext {
        crate::app::tests::TestAppBuilder::new(prefix).build().await
    }
}
