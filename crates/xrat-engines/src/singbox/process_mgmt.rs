use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use xrat_support::process::{Command, StartupChild, Stdio};

use thiserror::Error;

use crate::singbox::SingboxConfig;

#[derive(Debug, Error)]
pub enum SingboxRuntimeError {
    #[error("failed to create runtime files: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize sing-box config: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to spawn sing-box process: {0}")]
    Spawn(String),
    #[error("sing-box process exited during startup: {0}")]
    ProcessExited(String),
    #[error("sing-box did not open local port {port} before startup timeout")]
    StartupTimeout { port: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedSingboxPaths {
    pub config_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedSingboxProcess {
    pub pid: u32,
    pub ready_port: u16,
    pub paths: ManagedSingboxPaths,
}

pub async fn spawn_detached(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &SingboxConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
) -> Result<ManagedSingboxProcess, SingboxRuntimeError> {
    spawn_detached_with_ports(
        binary_path,
        runtime_dir,
        session_id,
        config,
        ready_host,
        ready_port,
        startup_timeout,
        xrat_support::readiness::RuntimeProcessPorts::default(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_detached_with_ports(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &SingboxConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
    ports: xrat_support::readiness::RuntimeProcessPorts,
) -> Result<ManagedSingboxProcess, SingboxRuntimeError> {
    std::fs::create_dir_all(runtime_dir)?;

    let paths = ManagedSingboxPaths {
        config_path: runtime_dir.join(format!("session-{session_id}.singbox.json")),
        stdout_path: runtime_dir.join(format!("session-{session_id}.singbox.out.log")),
        stderr_path: runtime_dir.join(format!("session-{session_id}.singbox.err.log")),
    };

    let mut config_file = File::create(&paths.config_path)?;
    config_file.write_all(serde_json::to_string_pretty(config)?.as_bytes())?;
    config_file.flush()?;

    let stdout = File::create(&paths.stdout_path)?;
    let stderr = File::create(&paths.stderr_path)?;
    let child = Command::with_spawner(binary_path, ports.spawner.clone())
        .arg("run")
        .arg("-c")
        .arg(&paths.config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|error| SingboxRuntimeError::Spawn(error.to_string()))?;

    let pid = child.id();
    let mut startup = StartupChild::new(child);
    match wait_for_ready(
        startup.child_mut(),
        ready_host,
        ready_port,
        startup_timeout,
        ports.waiter.as_ref(),
    )
    .await
    {
        Ok(()) => {
            startup.detach();
            Ok(ManagedSingboxProcess {
                pid,
                ready_port,
                paths,
            })
        }
        Err(error) => {
            drop(startup);
            Err(error.with_process_stderr(&paths.stderr_path))
        }
    }
}

async fn wait_for_ready(
    child: &mut xrat_support::process::Child,
    host: &str,
    port: u16,
    timeout: Duration,
    waiter: &dyn xrat_support::readiness::PortWaiter,
) -> Result<(), SingboxRuntimeError> {
    use xrat_support::readiness::{ReadinessError, ReadinessRequest};
    waiter
        .wait(child, ReadinessRequest::single(host, port, timeout))
        .await
        .map_err(|error| match error {
            ReadinessError::Io(error) => SingboxRuntimeError::Io(error),
            ReadinessError::ProcessExited(status) => {
                SingboxRuntimeError::ProcessExited(status.to_string())
            }
            ReadinessError::Timeout { port } => SingboxRuntimeError::StartupTimeout { port },
        })
}

trait StartupErrorExt {
    fn with_process_stderr(self, stderr_path: &Path) -> Self;
}

impl StartupErrorExt for SingboxRuntimeError {
    fn with_process_stderr(self, stderr_path: &Path) -> Self {
        let Some(stderr_tail) = read_stderr_tail(stderr_path) else {
            return self;
        };

        match self {
            SingboxRuntimeError::ProcessExited(status) => {
                SingboxRuntimeError::ProcessExited(format!("{status}; stderr: {stderr_tail}"))
            }
            SingboxRuntimeError::StartupTimeout { port } => SingboxRuntimeError::Spawn(format!(
                "sing-box did not open local port {port} before startup timeout; stderr: {stderr_tail}"
            )),
            other => other,
        }
    }
}

fn read_stderr_tail(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut output = String::new();
    file.read_to_string(&mut output).ok()?;
    let output = output.trim();
    if output.is_empty() {
        return None;
    }

    let mut lines: Vec<&str> = output.lines().rev().take(6).collect();
    lines.reverse();
    Some(lines.join(" | "))
}
