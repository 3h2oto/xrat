use crate::app::context::AppContext;
use crate::cli::ServeArgs;

pub async fn run(context: &AppContext, args: &ServeArgs) -> crate::app::Result<()> {
    let settings = resolve_settings(context, args)?;

    crate::server::serve_context(context, &settings, None).await
}

pub async fn webui(context: &AppContext, args: &crate::cli::WebUiArgs) -> crate::app::Result<()> {
    let mut context = context.clone();
    context.app_config.server.web.enabled = true;
    let settings = resolve_settings(&context, &args.serve)?;
    let origin = settings
        .web
        .origin(&settings.host, settings.port)
        .map_err(crate::app::AppError::InvalidArgument)?;
    if !args.no_open {
        let url = origin.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(750)).await;
            let (command, arguments): (&str, Vec<&str>) = if cfg!(target_os = "macos") {
                ("open", vec![url.as_str()])
            } else if cfg!(target_os = "windows") {
                (
                    "rundll32",
                    vec!["url.dll,FileProtocolHandler", url.as_str()],
                )
            } else {
                ("xdg-open", vec![url.as_str()])
            };
            if let Err(error) = tokio::process::Command::new(command)
                .args(arguments)
                .spawn()
            {
                tracing::warn!(%error, "could not open browser");
            }
        });
    }
    println!("XRAT Web UI: {origin}");
    crate::server::serve_context(&context, &settings, None).await
}

fn resolve_settings(
    context: &AppContext,
    args: &ServeArgs,
) -> crate::app::Result<crate::app::config::ServerSettings> {
    let config = &context.app_config.server;
    let mut settings = config.clone();
    if let Some(host) = &args.host {
        settings.host = host.clone();
    }
    if let Some(port) = args.port {
        settings.port = port;
    } else if std::fs::read_to_string(&context.runtime_paths.config_path)
        .ok()
        .and_then(|text| text.parse::<toml::Value>().ok())
        .and_then(|value| {
            value
                .get("server")
                .and_then(|server| server.get("port"))
                .cloned()
        })
        .is_none()
        && let Ok(port) = std::env::var("PORT")
    {
        settings.port = port.parse().map_err(|_| {
            crate::app::AppError::InvalidArgument("PORT must be a valid port".into())
        })?;
    }
    Ok(settings)
}

pub fn password_hash() -> crate::app::Result<()> {
    use std::io::Read;
    let mut password = String::new();
    std::io::stdin().take(1026).read_to_string(&mut password)?;
    let hash = crate::server::hash_password(password.trim_end_matches(['\r', '\n']))
        .map_err(crate::app::AppError::InvalidArgument)?;
    println!("{hash}");
    Ok(())
}
