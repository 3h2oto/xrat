use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::xray::XrayConfig;

#[derive(Debug, Error)]
pub enum XrayRuntimeError {
    #[error("failed to prepare Xray runtime files: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize Xray runtime config: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to spawn Xray process: {0}")]
    Spawn(String),
    #[error("Xray process exited during startup: {0}")]
    ProcessExited(String),
    #[error("Xray did not open local port {port} before startup timeout")]
    StartupTimeout { port: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedXrayPaths {
    pub config_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedXrayProcess {
    pub pid: u32,
    pub ready_port: u16,
    pub paths: ManagedXrayPaths,
}

pub async fn spawn_detached(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &XrayConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
) -> Result<ManagedXrayProcess, XrayRuntimeError> {
    std::fs::create_dir_all(runtime_dir)?;

    let paths = ManagedXrayPaths {
        config_path: runtime_dir.join(format!("session-{session_id}.json")),
        stdout_path: runtime_dir.join(format!("session-{session_id}.out.log")),
        stderr_path: runtime_dir.join(format!("session-{session_id}.err.log")),
    };

    let mut config_file = File::create(&paths.config_path)?;
    config_file.write_all(serde_json::to_string_pretty(config)?.as_bytes())?;
    config_file.flush()?;

    let stdout = File::create(&paths.stdout_path)?;
    let stderr = File::create(&paths.stderr_path)?;
    let mut command = Command::new(binary_path);
    configure_asset_path(&mut command, binary_path);
    let mut child = command
        .arg("run")
        .arg("-c")
        .arg(&paths.config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|error| XrayRuntimeError::Spawn(error.to_string()))?;

    let pid = child.id();
    match wait_for_ready(&mut child, ready_host, ready_port, startup_timeout).await {
        Ok(()) => Ok(ManagedXrayProcess {
            pid,
            ready_port,
            paths,
        }),
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(error.with_process_stderr(&paths.stderr_path))
        }
    }
}

fn configure_asset_path(command: &mut Command, binary_path: &Path) {
    let Some(directory) = crate::support::platform::managed_core_asset_dir(binary_path) else {
        return;
    };
    let variable = if binary_path.file_name().and_then(|name| name.to_str()) == Some("v2ray") {
        "V2RAY_LOCATION_ASSET"
    } else {
        "XRAY_LOCATION_ASSET"
    };
    command.env(variable, directory);
}

async fn wait_for_ready(
    child: &mut std::process::Child,
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<(), XrayRuntimeError> {
    let start = Instant::now();
    let check_interval = Duration::from_millis(100);
    let address = format!("{host}:{port}");

    loop {
        if let Some(status) = child.try_wait()? {
            return Err(XrayRuntimeError::ProcessExited(status.to_string()));
        }

        if TcpStream::connect(&address).await.is_ok() {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(XrayRuntimeError::StartupTimeout { port });
        }

        sleep(check_interval).await;
    }
}

pub fn process_is_running(pid: i64) -> bool {
    if pid <= 0 {
        return false;
    }

    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

trait StartupErrorExt {
    fn with_process_stderr(self, stderr_path: &Path) -> Self;
}

impl StartupErrorExt for XrayRuntimeError {
    fn with_process_stderr(self, stderr_path: &Path) -> Self {
        let Some(stderr_tail) = read_stderr_tail(stderr_path) else {
            return self;
        };

        match self {
            XrayRuntimeError::ProcessExited(status) => {
                XrayRuntimeError::ProcessExited(format!("{status}; stderr: {stderr_tail}"))
            }
            XrayRuntimeError::StartupTimeout { port } => XrayRuntimeError::Spawn(format!(
                "Xray did not open local port {port} before startup timeout; stderr: {stderr_tail}"
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
