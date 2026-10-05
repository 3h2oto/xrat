use crate::app::AppError;
use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::tun_privileges::{self, TunPrivilegeFile};
use crate::cli::{TunAction, TunArgs, TunSetupArgs, TunStatusArgs};
use xrat_support::process::Stdio;

pub async fn run(context: &AppContext, args: &TunArgs) -> crate::app::Result<()> {
    match &args.action {
        TunAction::Status(status) => status_command(context, status),
        TunAction::Setup(setup) => setup_command(context, setup),
    }
}

fn status_command(context: &AppContext, args: &TunStatusArgs) -> crate::app::Result<()> {
    let runtime = &context.app_config.runtime;
    let files = tun_privileges::required_files(context);
    let color = output::color_enabled();
    let supported = tun_privileges::capabilities_supported();

    let service_state = tun_privileges::systemd_service_status();
    let service_ready = service_state.as_ref().map(|(_, r)| *r).unwrap_or(true);

    if args.json {
        let entries: Vec<serde_json::Value> = files
            .iter()
            .map(|file| {
                let capabilities = tun_privileges::file_capabilities(&file.path);
                serde_json::json!({
                    "role": file.label,
                    "path": file.path.display().to_string(),
                    "capabilities": capabilities,
                    "ready": capabilities
                        .as_deref()
                        .map(tun_privileges::has_net_admin)
                        .unwrap_or(false),
                })
            })
            .collect();
        let all_files_ready = entries
            .iter()
            .all(|e| e["ready"].as_bool().unwrap_or(false));
        let value = serde_json::json!({
            "engine": runtime.engine.clone(),
            "tun_enabled": runtime.tun.enabled,
            "interface": runtime.tun.interface_name.clone(),
            "capabilities_supported": supported,
            "service_ready": service_ready,
            "ready": all_files_ready && service_ready,
            "files": entries,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    let mut rows: Vec<(&str, String)> = vec![
        ("engine", runtime.engine.clone()),
        (
            "tun enabled",
            if runtime.tun.enabled { "yes" } else { "no" }.to_string(),
        ),
        ("interface", runtime.tun.interface_name.clone()),
    ];
    if let Some((desc, _ready)) = &service_state {
        rows.push(("systemd service", desc.to_string()));
    }
    let mut missing = !service_ready;
    for file in &files {
        let (state, ready) = capability_state(file);
        missing |= !ready;
        rows.push((file.label, format!("{}  {state}", file.path.display())));
    }
    println!(
        "{}",
        output::format_kv(Some("TUN privileges"), &rows, color)
    );

    if !supported {
        println!(
            "{}",
            output::notice(
                "file capabilities are Linux-only; configure TUN privileges manually on this platform",
                color,
            )
        );
    } else if missing {
        println!(
            "{}",
            output::notice("run `xrat tun setup` to grant CAP_NET_ADMIN", color)
        );
    }
    Ok(())
}

fn capability_state(file: &TunPrivilegeFile) -> (String, bool) {
    if !file.path.is_file() {
        return (
            "not found; install the core or set [paths] to an absolute path".to_string(),
            false,
        );
    }
    match tun_privileges::file_capabilities(&file.path) {
        None => ("unknown (getcap unavailable)".to_string(), true),
        Some(capabilities) if tun_privileges::has_net_admin(&capabilities) => {
            (format!("ready ({capabilities})"), true)
        }
        Some(capabilities) if capabilities.is_empty() => {
            ("missing (run `xrat tun setup`)".to_string(), false)
        }
        Some(capabilities) => (format!("missing CAP_NET_ADMIN ({capabilities})"), false),
    }
}

fn setup_command(context: &AppContext, args: &TunSetupArgs) -> crate::app::Result<()> {
    if !tun_privileges::capabilities_supported() {
        return Err(AppError::InvalidArgument(
            "file capabilities are Linux-only; configure TUN privileges manually on this platform"
                .to_string(),
        ));
    }
    let files = tun_privileges::required_files(context);
    let color = output::color_enabled();

    let Some(engine_file) = files.first() else {
        return Err(AppError::InvalidArgument(
            "could not determine the engine binary path".to_string(),
        ));
    };
    if !engine_file.path.is_file() {
        return Err(AppError::InvalidArgument(format!(
            "engine binary not found at {}; install it (for example `xrat install xray`) or set [paths] to an absolute path",
            engine_file.path.display()
        )));
    }
    let files: Vec<TunPrivilegeFile> = files
        .into_iter()
        .filter(|file| file.path.is_file())
        .collect();

    let commands: Vec<String> = files
        .iter()
        .map(|file| {
            format!(
                "sudo setcap {} {}",
                tun_privileges::TUN_CAPABILITIES,
                file.path.display()
            )
        })
        .collect();

    if args.dry_run {
        for command in &commands {
            println!("{command}");
        }
        return Ok(());
    }

    println!(
        "{}",
        output::notice("granting TUN capabilities with `sudo setcap`", color)
    );
    // `setcap` accepts a single file per capability specification, so each file
    // gets its own invocation (sudo caches the credential across them).
    for (file, printable) in files.iter().zip(&commands) {
        let mut command = xrat_support::process::Command::new("sudo");
        command
            .arg("setcap")
            .arg(tun_privileges::TUN_CAPABILITIES)
            .arg(&file.path);
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let status = command.status()?;
        if !status.success() {
            return Err(AppError::InvalidArgument(format!(
                "`{printable}` failed ({status}); run it manually and check your privileges"
            )));
        }
    }

    println!("{}", output::success("Granted TUN capabilities.", color));

    #[cfg(target_os = "linux")]
    {
        if let Ok(user_dir) = crate::app::commands::daemon_install::systemd_user_dir_with_env(
            &xrat_support::env::SystemEnvVars,
        ) {
            let service_file =
                user_dir.join(crate::app::commands::daemon_install::DAEMON_SERVICE_NAME);
            let override_dir =
                user_dir.join(crate::app::commands::daemon_install::DAEMON_OVERRIDE_DIR_NAME);
            let override_file = override_dir
                .join(crate::app::commands::daemon_install::DAEMON_TUN_OVERRIDE_FILE_NAME);

            if args.dry_run {
                if service_file.is_file() || user_dir.is_dir() {
                    println!("write {} (NoNewPrivileges=false)", override_file.display());
                    println!("systemctl --user daemon-reload");
                }
            } else if service_file.is_file() || user_dir.is_dir() {
                let _ = std::fs::create_dir_all(&override_dir);
                let _ = std::fs::write(
                    &override_file,
                    crate::app::commands::daemon_install::DAEMON_TUN_OVERRIDE_TEMPLATE,
                );
                let _ = xrat_support::process::Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .status();
            }
        }
    }

    if !args.dry_run {
        let mut rows: Vec<(&str, String)> = Vec::new();
        for file in &files {
            let (state, _) = capability_state(file);
            rows.push((file.label, format!("{}  {state}", file.path.display())));
        }
        if let Some((desc, _)) = tun_privileges::systemd_service_status() {
            rows.push(("systemd service", desc.to_string()));
        }
        println!("{}", output::format_kv(None, &rows, color));
        println!(
            "{}",
            output::notice(
                "restart the daemon (`xrat daemon restart`) so it picks up capabilities on the xrat binary",
                color,
            )
        );
    }
    Ok(())
}
