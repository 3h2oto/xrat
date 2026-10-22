use super::{Transaction, save};
use crate::app::{
    AppError, Result,
    services::machine_service::{MachineProfile, api_task, scm},
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use windows_sys::Win32::System::Services::{SERVICE_RUNNING, SERVICE_STOPPED};
use xrat_support::windows::ProtectedDirectory;

pub(crate) fn machine_target() -> Result<PathBuf> {
    Ok(xrat_support::windows::program_files()?.join("XRAT-Service/xrat.exe"))
}
pub(crate) fn classify(target: &Path, config: &Path) -> Result<bool> {
    let expected = machine_target()?;
    let machine = expected.try_exists()? && target.canonicalize()? == expected.canonicalize()?;
    if machine {
        if !xrat_support::windows::is_current_process_elevated()? {
            return Err(AppError::InvalidArgument(
                "machine upgrade requires elevation".into(),
            ));
        }
        let profile = MachineProfile::open()?;
        if config != profile.config_path() {
            return Err(AppError::InvalidArgument(
                "use --profile machine upgrade for the protected service executable".into(),
            ));
        }
        ProtectedDirectory::open_readable(expected.parent().unwrap(), profile.operator_sid())?
            .verify_tree()?;
        if scm::status()?.is_none() {
            return Err(AppError::InvalidArgument(
                "machine service is not registered".into(),
            ));
        }
    } else if xrat_support::windows::is_current_process_elevated()? {
        return Err(AppError::InvalidArgument("run user-profile upgrades without elevation; protected service upgrades use --profile machine".into()));
    }
    Ok(machine)
}
pub(super) async fn quiesce(manifest: &Path, transaction: &mut Transaction) -> Result<()> {
    if transaction.machine {
        let profile = MachineProfile::open()?;
        let status = scm::status()?.ok_or_else(|| {
            AppError::InvalidArgument("machine service disappeared before upgrade".into())
        })?;
        if !matches!(status.dwCurrentState, SERVICE_RUNNING | SERVICE_STOPPED) {
            return Err(AppError::InvalidArgument(
                "wait for the machine service lifecycle operation before upgrading".into(),
            ));
        }
        transaction.was_running = status.dwCurrentState == SERVICE_RUNNING;
        (transaction.api_enabled, transaction.api_running) =
            api_task::inspect(profile.operator_sid())?;
        transaction.coordination_started = true;
        save(manifest, transaction)?;
        api_task::pause(profile.operator_sid())?;
        scm::stop()?;
    } else {
        let runtime = crate::app::app_paths::resolve()?.root_dir.join("runtime");
        let endpoint = crate::app::daemon::ipc::default_socket_path(&runtime);
        transaction.was_running = crate::app::daemon::ipc::ping_daemon(&endpoint)
            .await
            .is_ok();
        transaction.daemon_endpoint = Some(endpoint.clone());
        transaction.coordination_started = true;
        save(manifest, transaction)?;
        if transaction.was_running {
            let response = crate::app::daemon::ipc::daemon_shutdown_daemon(&endpoint).await?;
            if !response.ok {
                return Err(AppError::InvalidArgument(response.message));
            }
            let deadline = tokio::time::Instant::now() + Duration::from_secs(25);
            while crate::app::daemon::ipc::ping_daemon(&endpoint)
                .await
                .is_ok()
            {
                if tokio::time::Instant::now() >= deadline {
                    return Err(AppError::InvalidArgument(
                        "user daemon did not stop; executable preserved".into(),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    if !transaction.machine {
        let status = xrat_support::process::Command::new(&transaction.target)
            .arg("--config")
            .arg(&transaction.config)
            .arg("disconnect")
            .status()?;
        if !status.success() {
            return Err(AppError::InvalidArgument(
                "user runtime cleanup failed; executable preserved".into(),
            ));
        }
    }
    Ok(())
}
pub(super) async fn migrate_and_resume(transaction: &Transaction) -> Result<()> {
    if transaction.machine {
        // Service startup applies its own protected database migrations as SYSTEM.
        scm::start()?;
        if !transaction.was_running || !transaction.restart_daemon {
            scm::stop()?;
        }
        let profile = MachineProfile::open()?;
        api_task::resume(
            profile.operator_sid(),
            transaction.api_enabled,
            transaction.api_running && transaction.restart_daemon,
        )?;
    } else {
        super::super::run_post_upgrade_migrations(&transaction.target, &transaction.config)?;
        resume_user(transaction).await?;
    }
    Ok(())
}
async fn resume_user(transaction: &Transaction) -> Result<()> {
    if transaction.was_running && transaction.restart_daemon {
        let status = xrat_support::process::Command::new(&transaction.target)
            .arg("--config")
            .arg(&transaction.config)
            .args(["daemon", "start"])
            .status()?;
        if !status.success() {
            return Err(AppError::InvalidArgument(format!(
                "updated user daemon did not restart ({status})"
            )));
        }
    }
    Ok(())
}
pub(super) async fn recover_before_install(transaction: &Transaction) -> Result<()> {
    if !transaction.coordination_started {
        return Ok(());
    }
    if transaction.machine {
        if transaction.was_running {
            scm::start()?;
        }
        let profile = MachineProfile::open()?;
        api_task::resume(
            profile.operator_sid(),
            transaction.api_enabled,
            transaction.api_running,
        )?;
        Ok(())
    } else {
        let mut recovery = transaction.clone();
        recovery.restart_daemon = true;
        resume_user(&recovery).await
    }
}

pub(crate) fn is_service_path(path: &Path) -> Result<bool> {
    let target = machine_target()?;
    let root = target.parent().unwrap();
    if !root.try_exists()? {
        return Ok(false);
    }
    let path = if path.try_exists()? {
        path.canonicalize()?
    } else {
        path.parent()
            .ok_or_else(|| AppError::InvalidArgument("output has no parent".into()))?
            .canonicalize()?
            .join(
                path.file_name()
                    .ok_or_else(|| AppError::InvalidArgument("output has no name".into()))?,
            )
    };
    Ok(path.starts_with(root.canonicalize()?))
}
pub(crate) fn open_directory(path: &Path) -> Result<ProtectedDirectory> {
    if path.canonicalize()? == machine_target()?.parent().unwrap().canonicalize()? {
        let profile = MachineProfile::open()?;
        Ok(ProtectedDirectory::open_readable(
            path,
            profile.operator_sid(),
        )?)
    } else {
        Ok(ProtectedDirectory::open(path)?)
    }
}
