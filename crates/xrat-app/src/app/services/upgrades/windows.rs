use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::UpgradeOptions;
use crate::app::{AppError, Result};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum State {
    Staged,
    Applying,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    target: PathBuf,
    config: PathBuf,
    parent_pid: u32,
    parent_creation: u64,
    digest: String,
    version: String,
    state: State,
    message: String,
    restart_daemon: bool,
}

fn save(path: &Path, transaction: &Transaction) -> Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(
        path.parent()
            .ok_or_else(|| AppError::InvalidArgument("upgrade manifest has no parent".into()))?,
    )?;
    file.write_all(&serde_json::to_vec_pretty(transaction)?)?;
    file.as_file().sync_all()?;
    file.persist(path)
        .map_err(|error| AppError::Io(error.error))?;
    Ok(())
}

fn digest(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn lock(directory: &Path, wait: bool) -> Result<File> {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(directory.join("xrat-upgrade.lock"))?;
    if wait {
        file.lock()?;
    } else {
        file.try_lock().map_err(|error| {
            AppError::InvalidArgument(format!("another upgrade is active: {error}"))
        })?;
    }
    Ok(file)
}

pub(super) async fn stage(
    binary: &Path,
    target: &Path,
    config: &Path,
    version: &str,
    args: &UpgradeOptions,
) -> Result<PathBuf> {
    let target = target.canonicalize()?;
    let directory = target
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("upgrade target has no parent".into()))?;
    let _lock = lock(directory, false)?;
    let record = directory.join("xrat-upgrade.json");
    if record.exists() {
        let previous: PathBuf = serde_json::from_slice(&fs::read(&record)?)?;
        let previous: Transaction = serde_json::from_slice(&fs::read(previous)?)?;
        if matches!(previous.state, State::Staged | State::Applying) {
            return Err(AppError::InvalidArgument("an earlier upgrade is still staged or applying; inspect xrat upgrade --status before retrying".into()));
        }
    }
    let work = tempfile::Builder::new()
        .prefix(".xrat-upgrade-")
        .tempdir_in(directory)
        .map_err(|error| {
            AppError::InvalidArgument(format!(
                "cannot stage upgrade in {}; protected installation requires elevation: {error}",
                directory.display()
            ))
        })?;
    let next = work.path().join("next.exe");
    fs::copy(binary, &next)?;
    let check = xrat_support::process::Command::new(&next)
        .arg("--version")
        .output()?;
    let reported = String::from_utf8_lossy(&check.stdout);
    let reported = reported.trim().strip_prefix("xrat ").ok_or_else(|| {
        AppError::InvalidArgument("staged executable did not report an XRAT version".into())
    })?;
    if !check.status.success() || (version != "source" && !super::same_version(reported, version)) {
        return Err(AppError::InvalidArgument(
            "staged executable version does not match the requested release".into(),
        ));
    }
    let runtime = crate::app::app_paths::resolve()?.root_dir.join("runtime");
    let endpoint = crate::app::daemon::ipc::default_socket_path(&runtime);
    if crate::app::daemon::ipc::ping_daemon(&endpoint)
        .await
        .is_ok()
    {
        return Err(AppError::InvalidArgument(
            "stop the running daemon before upgrading its executable; no files were replaced"
                .into(),
        ));
    }
    let transaction = Transaction {
        target: target.clone(),
        config: std::path::absolute(config)?,
        parent_pid: std::process::id(),
        parent_creation: xrat_support::windows::process_creation_time(std::process::id())?,
        digest: digest(&next)?,
        version: reported.to_string(),
        state: State::Staged,
        message:
            "Exit the upgrading XRAT process within 120 seconds; then run xrat upgrade --status"
                .into(),
        restart_daemon: !args.no_restart_daemon,
    };
    let manifest = work.path().join("transaction.json");
    save(&manifest, &transaction)?;
    let helper = work.path().join("updater.exe");
    fs::copy(std::env::current_exe()?, &helper)?;
    fs::write(&record, serde_json::to_vec(&manifest)?)?;
    xrat_support::process::Command::new(&helper)
        .args(["upgrade", "--apply-transaction"])
        .arg(&manifest)
        .detached(true)
        .stdin(xrat_support::process::Stdio::null())
        .stdout(xrat_support::process::Stdio::null())
        .stderr(xrat_support::process::Stdio::null())
        .spawn()?;
    let _ = work.keep();
    Ok(manifest)
}

pub(crate) async fn apply_transaction(manifest: &Path) -> Result<()> {
    let manifest = manifest.canonicalize()?;
    let directory = manifest
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("upgrade manifest has no parent".into()))?;
    if std::env::current_exe()?.canonicalize()? != directory.join("updater.exe").canonicalize()? {
        return Err(AppError::InvalidArgument(
            "upgrade helper must execute from the staged transaction directory".into(),
        ));
    }
    let mut transaction: Transaction = serde_json::from_slice(&fs::read(&manifest)?)?;
    let _lock = lock(
        directory
            .parent()
            .ok_or_else(|| AppError::InvalidArgument("upgrade directory has no parent".into()))?,
        true,
    )?;
    if !matches!(transaction.state, State::Staged)
        || transaction.target.parent() != directory.parent()
    {
        return Err(AppError::InvalidArgument(
            "invalid upgrade transaction state or target".into(),
        ));
    }
    let result = apply(&manifest, &mut transaction).await;
    if let Err(error) = &result {
        if !matches!(transaction.state, State::RolledBack) {
            transaction.state = State::Failed;
        }
        transaction.message = error.to_string();
        save(&manifest, &transaction)?;
    }
    result
}

async fn apply(manifest: &Path, transaction: &mut Transaction) -> Result<()> {
    xrat_support::windows::wait_process_exit(
        transaction.parent_pid,
        transaction.parent_creation,
        Duration::from_secs(120),
    )?;
    let directory = manifest
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("upgrade manifest has no parent".into()))?;
    let next = directory.join("next.exe");
    if digest(&next)? != transaction.digest {
        return Err(AppError::InvalidArgument(
            "staged executable checksum changed; original installation preserved".into(),
        ));
    }
    transaction.state = State::Applying;
    save(manifest, transaction)?;
    let backup = directory.join("backup.exe");
    fs::copy(&transaction.target, &backup)?;
    if let Err(error) = super::install_binary(&next, &transaction.target) {
        transaction.state = State::RolledBack;
        return Err(AppError::InvalidArgument(format!(
            "replacement failed; original executable preserved: {error}"
        )));
    }
    if let Err(error) = super::run_post_upgrade_migrations(&transaction.target, &transaction.config)
    {
        return Err(AppError::InvalidArgument(format!(
            "{error}; new executable retained because migrations may have changed the database; binary backup: {}",
            backup.display()
        )));
    }
    fs::remove_file(backup)?;
    fs::remove_file(next)?;
    transaction.state = State::Completed;
    transaction.message = format!(
        "XRAT {} installed and database migrations completed",
        transaction.version
    );
    save(manifest, transaction)?;
    Ok(())
}

pub(crate) fn status() -> Result<String> {
    let executable = std::env::current_exe()?.canonicalize()?;
    let parent = executable
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("executable has no parent".into()))?;
    let manifest: PathBuf = serde_json::from_slice(&fs::read(parent.join("xrat-upgrade.json"))?)?;
    let manifest = manifest.canonicalize()?;
    let directory = manifest
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("upgrade manifest has no parent".into()))?;
    if directory.parent() != Some(parent)
        || !directory
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".xrat-upgrade-"))
    {
        return Err(AppError::InvalidArgument(
            "upgrade record points outside this installation".into(),
        ));
    }
    let transaction: Transaction = serde_json::from_slice(&fs::read(&manifest)?)?;
    if transaction.target != executable {
        return Err(AppError::InvalidArgument(
            "upgrade record belongs to a different executable".into(),
        ));
    }
    if matches!(transaction.state, State::Completed) {
        let helper = manifest
            .parent()
            .unwrap_or(Path::new("."))
            .join("updater.exe");
        match fs::remove_file(helper) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::debug!(%error, "upgrade helper cleanup is pending"),
        }
    }
    Ok(serde_json::to_string_pretty(&transaction)?)
}
