#[path = "windows/coordination.rs"]
pub(super) mod coordination;
#[path = "windows/ordinary.rs"]
mod ordinary;

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::UpgradeOptions;
use crate::app::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum State {
    Staged,
    Applying,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transaction {
    #[serde(default)]
    machine: bool,
    #[serde(default)]
    was_running: bool,
    #[serde(default)]
    daemon_endpoint: Option<PathBuf>,
    #[serde(default)]
    api_enabled: bool,
    #[serde(default)]
    api_running: bool,
    #[serde(default)]
    coordination_started: bool,
    #[serde(default)]
    binary_installed: bool,
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
    let mut file = super::staging::StagedFile::new(
        path.parent()
            .ok_or_else(|| AppError::InvalidArgument("upgrade manifest has no parent".into()))?,
    )?;
    file.as_file_mut()
        .write_all(&serde_json::to_vec_pretty(transaction)?)?;
    file.as_file().sync_all()?;
    file.persist(path)?;
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    use std::io::Read;
    let mut bytes = Vec::new();
    File::open(path)?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(AppError::InvalidArgument(
            "upgrade record exceeds 64 KiB".into(),
        ));
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn digest(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn lock(directory: &Path, wait: bool) -> Result<File> {
    if coordination::is_service_path(directory)? {
        let guard = coordination::open_directory(directory)?;
        guard.verify_tree()?;
        match guard.create_file("xrat-upgrade.lock") {
            Ok(file) => drop(file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(directory.join("xrat-upgrade.lock"))?;
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        match file.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock)
                if wait && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                return Err(AppError::InvalidArgument(format!(
                    "another upgrade is active or its lock is unavailable: {error}"
                )));
            }
        }
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
    let machine = coordination::classify(&target, config)?;
    let _lock = lock(directory, false)?;
    let record = directory.join("xrat-upgrade.json");
    if record.exists() {
        let previous: PathBuf = read_json(&record)?;
        let previous: Transaction = read_json(&previous)?;
        if matches!(previous.state, State::Staged | State::Applying) {
            return Err(AppError::InvalidArgument("an earlier upgrade is still staged or applying; inspect xrat upgrade --status before retrying".into()));
        }
    }
    let work = Work::new(directory, machine, ".xrat-upgrade-")?;
    let next = work.path().join("next.exe");
    copy_file(binary, &next)?;
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
    let transaction = Transaction {
        machine,
        was_running: false,
        daemon_endpoint: None,
        api_enabled: false,
        api_running: false,
        coordination_started: false,
        binary_installed: false,
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
        restart_daemon: super::should_restart_daemon(config, args),
    };
    let manifest = work.path().join("transaction.json");
    save(&manifest, &transaction)?;
    let helper = work.path().join("updater.exe");
    copy_file(&std::env::current_exe()?, &helper)?;
    write_file(&record, &serde_json::to_vec(&manifest)?)?;
    work.keep();
    let spawned = xrat_support::process::Command::new(&helper)
        .args(["upgrade", "--apply-transaction"])
        .arg(&manifest)
        .detached(true)
        .stdin(xrat_support::process::Stdio::null())
        .stdout(xrat_support::process::Stdio::null())
        .stderr(xrat_support::process::Stdio::null())
        .spawn();
    if let Err(error) = spawned {
        let mut failed = transaction;
        failed.state = State::Failed;
        failed.message = format!("could not start updater; installation preserved: {error}");
        save(&manifest, &failed)?;
        return Err(error.into());
    }
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
    let mut transaction: Transaction = read_json(&manifest)?;
    if !transaction.machine && xrat_support::windows::is_current_process_elevated()? {
        return ordinary::apply(&manifest).await;
    }
    coordination::classify(&transaction.target, &transaction.config).and_then(|machine| {
        if machine == transaction.machine {
            Ok(())
        } else {
            Err(AppError::InvalidArgument(
                "upgrade transaction scope mismatch".into(),
            ))
        }
    })?;
    if transaction.machine {
        coordination::open_directory(directory)?.verify_tree()?;
    }
    let _lock = lock(
        directory
            .parent()
            .ok_or_else(|| AppError::InvalidArgument("upgrade directory has no parent".into()))?,
        true,
    )?;
    if !matches!(transaction.state, State::Staged)
        || transaction.target.parent() != directory.parent()
        || !directory
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".xrat-upgrade-"))
    {
        return Err(AppError::InvalidArgument(
            "invalid upgrade transaction state or target".into(),
        ));
    }
    let result = apply(&manifest, &mut transaction).await;
    if let Err(error) = &result {
        if !transaction.binary_installed
            && let Err(recovery) = coordination::recover_before_install(&transaction).await
        {
            transaction.message =
                format!("{error}; restoring previous service state failed: {recovery}");
        }
        if !matches!(transaction.state, State::RolledBack) {
            transaction.state = State::Failed;
        }
        if transaction.message.is_empty()
            || !transaction
                .message
                .contains("restoring previous service state failed")
        {
            transaction.message = error.to_string();
        }
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
    coordination::quiesce(manifest, transaction).await?;
    let profile = if transaction.machine {
        Some(crate::app::services::machine_service::MachineProfile::open()?)
    } else {
        None
    };
    let lease = profile
        .as_ref()
        .map(|profile| profile.lifecycle_lease())
        .transpose()?;
    let backup = directory.join("backup.exe");
    copy_file(&transaction.target, &backup)?;
    let replacement = {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            match super::install_binary(&next, &transaction.target) {
                Ok(()) => break Ok(()),
                Err(error) if std::time::Instant::now() >= deadline => break Err(error),
                Err(_) => std::thread::sleep(Duration::from_millis(100)),
            }
        }
    };
    if let Err(error) = replacement {
        transaction.state = State::RolledBack;
        return Err(AppError::InvalidArgument(format!(
            "replacement failed; original executable preserved: {error}"
        )));
    }
    transaction.binary_installed = true;
    save(manifest, transaction)?;
    drop(lease);
    if let Err(error) = coordination::migrate_and_resume(transaction).await {
        return Err(AppError::InvalidArgument(format!(
            "{error}; updated executable and backup retained because database migrations may have run; the machine API task may remain paused, inspect {} and restore service/task state after recovery",
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
    status_for(&std::env::current_exe()?)
}
pub(crate) fn status_for(executable: &Path) -> Result<String> {
    let executable = executable.canonicalize()?;
    let parent = executable
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("executable has no parent".into()))?;
    let manifest: PathBuf = read_json(&parent.join("xrat-upgrade.json"))?;
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
    let transaction: Transaction = read_json(&manifest)?;
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

pub(super) struct Work {
    path: PathBuf,
    temporary: Option<tempfile::TempDir>,
    protected: Option<xrat_support::windows::ProtectedDirectory>,
    retained: bool,
}
impl Work {
    pub fn new(directory: &Path, machine: bool, prefix: &str) -> Result<Self> {
        if machine {
            let path = directory.join(format!("{prefix}{}", uuid::Uuid::new_v4()));
            let protected = xrat_support::windows::ProtectedDirectory::create(&path)?;
            Ok(Self {
                path,
                temporary: None,
                protected: Some(protected),
                retained: false,
            })
        } else {
            let temporary = tempfile::Builder::new()
                .prefix(prefix)
                .tempdir_in(directory)?;
            Ok(Self {
                path: temporary.path().to_path_buf(),
                temporary: Some(temporary),
                protected: None,
                retained: false,
            })
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn keep(mut self) {
        self.retained = true;
        if let Some(temporary) = self.temporary.take() {
            let _ = temporary.keep();
        }
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        if self.protected.take().is_some() && !self.retained {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
pub(super) fn output_file(path: &Path) -> Result<File> {
    if coordination::is_service_path(path)? {
        let directory = coordination::open_directory(
            path.parent()
                .ok_or_else(|| AppError::InvalidArgument("output has no parent".into()))?,
        )?;
        directory.verify_tree()?;
        Ok(directory.create_file(
            path.file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| AppError::InvalidArgument("invalid output file name".into()))?,
        )?)
    } else {
        Ok(File::create(path)?)
    }
}
fn copy_file(source: &Path, target: &Path) -> Result<()> {
    let mut output = output_file(target)?;
    std::io::copy(&mut File::open(source)?, &mut output)?;
    output.sync_all()?;
    Ok(())
}
fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut staged = super::staging::StagedFile::new(
        path.parent()
            .ok_or_else(|| AppError::InvalidArgument("output has no parent".into()))?,
    )?;
    staged.as_file_mut().write_all(bytes)?;
    staged.as_file().sync_all()?;
    staged.persist(path)
}

#[cfg(test)]
#[path = "windows/native_tests.rs"]
mod native_tests;
#[cfg(test)]
pub(crate) use native_tests::exercise_machine_upgrade;
