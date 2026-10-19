use super::*;

pub async fn run(config_path: &Path, args: &UpgradeArgs) -> crate::app::Result<()> {
    let target = current_exe()?;

    if args.source {
        source::upgrade(args, &target, config_path).await
    } else {
        release::upgrade(args, &target, config_path).await
    }
}

/// Run database migrations using the freshly installed binary so any migration
/// failure is reported as part of `xrat upgrade` rather than surfacing on the
/// next unrelated command.
pub(crate) fn run_post_upgrade_migrations(
    target: &Path,
    config_path: &Path,
) -> crate::app::Result<()> {
    run_post_upgrade_migrations_with_spawner(
        target,
        config_path,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
    )
}

pub(crate) fn run_post_upgrade_migrations_with_spawner(
    target: &Path,
    config_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> crate::app::Result<()> {
    let status = Command::with_spawner(target, spawner.clone())
        .arg("--config")
        .arg(config_path)
        .arg("db")
        .arg("migrate")
        .status()
        .map_err(|error| {
            AppError::InvalidArgument(format!(
                "could not run post-upgrade database migrations: {error}"
            ))
        })?;

    if !status.success() {
        return Err(AppError::InvalidArgument(format!(
            "post-upgrade database migration failed ({status}); run `xrat db migrate` for details"
        )));
    }

    Ok(())
}

pub(crate) fn should_restore_tun(target: &Path, config_path: &Path, args: &UpgradeArgs) -> bool {
    if args.no_tun_setup {
        return false;
    }
    if args.tun_setup {
        return true;
    }

    #[cfg(target_os = "linux")]
    {
        let config = crate::app::config::load(config_path).ok();
        let mode = config
            .as_ref()
            .map(|c| c.runtime.tun.auto_setup_on_upgrade)
            .unwrap_or(crate::app::config::TunAutoSetupMode::Auto);

        match mode {
            crate::app::config::TunAutoSetupMode::Never => false,
            crate::app::config::TunAutoSetupMode::Always => true,
            crate::app::config::TunAutoSetupMode::Auto => {
                let target_has_caps = crate::app::tun_privileges::file_capabilities(target)
                    .as_deref()
                    .is_some_and(|caps| !caps.is_empty());
                let tun_enabled = config.as_ref().is_some_and(|c| c.runtime.tun.enabled);
                let override_exists = crate::app::tun_privileges::systemd_service_status()
                    .is_some_and(|(msg, _)| msg.contains("override present"));

                let engine_has_caps = config
                    .as_ref()
                    .and_then(|c| {
                        let binary = match c.runtime.engine.as_str() {
                            "sing-box" => {
                                c.paths.sing_box.as_deref().unwrap_or(Path::new("sing-box"))
                            }
                            _ => c.paths.xray.as_deref().unwrap_or(Path::new("xray")),
                        };
                        crate::app::tun_privileges::resolve_executable(binary)
                    })
                    .and_then(|path| crate::app::tun_privileges::file_capabilities(&path))
                    .is_some_and(|caps| !caps.is_empty());

                target_has_caps || engine_has_caps || tun_enabled || override_exists
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (target, config_path);
        false
    }
}

pub(crate) fn should_restart_daemon(config_path: &Path, args: &UpgradeArgs) -> bool {
    if args.no_restart_daemon {
        return false;
    }
    let config = crate::app::config::load(config_path).ok();
    config
        .as_ref()
        .is_none_or(|c| c.runtime.tun.restart_daemon_on_upgrade)
}

pub(crate) fn run_post_upgrade_tun_setup_with_spawner(
    target: &Path,
    config_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> bool {
    let color = output::color_enabled();
    println!(
        "{}",
        output::notice("restoring TUN capabilities with `tun setup`...", color)
    );

    let mut command = Command::with_spawner(target, spawner);
    command
        .arg("--config")
        .arg(config_path)
        .arg("tun")
        .arg("setup")
        .stdin(xrat_support::process::Stdio::inherit())
        .stdout(xrat_support::process::Stdio::inherit())
        .stderr(xrat_support::process::Stdio::inherit());

    match command.status() {
        Ok(status) if status.success() => {
            println!("{}", output::success("Restored TUN capabilities.", color));
            true
        }
        Ok(status) => {
            println!(
                "{}",
                output::warn(
                    format!(
                        "`xrat tun setup` exited with {status}; run it manually to restore TUN capabilities"
                    ),
                    color
                )
            );
            false
        }
        Err(err) => {
            println!(
                "{}",
                output::warn(
                    format!(
                        "could not run `xrat tun setup`: {err}; run it manually to restore TUN capabilities"
                    ),
                    color
                )
            );
            false
        }
    }
}

pub(crate) async fn restart_daemon_if_active(
    target: &Path,
    config_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) {
    #[cfg(target_os = "linux")]
    {
        let color = output::color_enabled();
        let is_systemd_active = Command::with_spawner("systemctl", spawner.clone())
            .args(["--user", "is-active", "--quiet", "xrat-daemon.service"])
            .status()
            .is_ok_and(|s| s.success());

        if is_systemd_active {
            let status = Command::with_spawner("systemctl", spawner)
                .args(["--user", "restart", "xrat-daemon.service"])
                .status();
            if status.is_ok_and(|s| s.success()) {
                println!(
                    "{}",
                    output::success("Restarted xrat-daemon.service.", color)
                );
            } else {
                println!(
                    "{}",
                    output::warn(
                        "Could not restart xrat-daemon.service; run `systemctl --user restart xrat-daemon.service` manually.",
                        color
                    )
                );
            }
            return;
        }
    }

    let runtime_dir = crate::app::app_paths::resolve()
        .ok()
        .map(|paths| paths.root_dir.join("runtime"))
        .unwrap_or_else(|| {
            config_path
                .parent()
                .unwrap_or(Path::new("."))
                .join("runtime")
        });
    let socket_path = crate::app::daemon::ipc::default_socket_path(&runtime_dir);
    restart_standalone_if_active(target, config_path, &socket_path, spawner).await;
}

pub(super) async fn restart_standalone_if_active(
    target: &Path,
    config_path: &Path,
    socket_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) {
    let color = output::color_enabled();
    if tokio::time::timeout(
        std::time::Duration::from_secs(2),
        crate::app::daemon::ipc::ping_daemon(socket_path),
    )
    .await
    .is_ok_and(|response| response.is_ok())
    {
        let status = Command::with_spawner(target, spawner)
            .arg("--config")
            .arg(config_path)
            .args(["daemon", "restart"])
            .status();
        if status.is_ok_and(|s| s.success()) {
            println!("{}", output::success("Restarted standalone daemon.", color));
        } else {
            println!(
                "{}",
                output::warn(
                    "Could not restart standalone daemon; run `xrat daemon restart` manually.",
                    color
                )
            );
        }
    }
}

pub(super) fn current_exe() -> crate::app::Result<PathBuf> {
    std::env::current_exe().map_err(|error| {
        AppError::InvalidArgument(format!(
            "could not resolve the running xrat binary: {error}"
        ))
    })
}

/// Atomically replace the running binary at `target` with `new_binary`.
///
/// The new file is staged in the same directory so the final rename stays on a
/// single filesystem; on Linux replacing a running executable this way is safe.
pub(crate) fn install_binary(new_binary: &Path, target: &Path) -> crate::app::Result<()> {
    let dir = target.parent().unwrap_or_else(|| Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(dir).map_err(|error| {
        AppError::InvalidArgument(format!(
            "cannot stage upgrade in {} (permission?): {error}",
            dir.display()
        ))
    })?;
    std::io::copy(&mut std::fs::File::open(new_binary)?, staged.as_file_mut())?;
    staged.as_file().sync_all()?;
    set_executable(staged.path())?;

    staged.persist(target).map_err(|error| {
        AppError::InvalidArgument(format!(
            "cannot replace {} (permission?): {}",
            target.display(),
            error.error
        ))
    })?;

    Ok(())
}

pub(super) fn set_executable(path: &Path) -> crate::app::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Compare release tags ignoring a leading `v`, e.g. `v0.2.1` == `0.2.1`.
pub(crate) fn same_version(left: &str, right: &str) -> bool {
    normalize_tag(left) == normalize_tag(right)
}

pub(super) fn normalize_tag(tag: &str) -> &str {
    tag.trim().trim_start_matches('v')
}

pub(crate) fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
