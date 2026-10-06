use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use xrat_support::process::{Child, Command, Stdio};

use tempfile::NamedTempFile;

use super::super::config::XrayConfig;
use super::errors::XrayProcessError;

pub struct XrayProcess {
    child: Child,
    config_file: NamedTempFile,
    local_port: u16,
}

impl XrayProcess {
    pub async fn spawn(
        config: &XrayConfig,
        startup_timeout: Duration,
    ) -> Result<Self, XrayProcessError> {
        Self::spawn_with_binary(Path::new("xray"), config, startup_timeout).await
    }

    pub async fn spawn_with_binary(
        binary_path: &Path,
        config: &XrayConfig,
        startup_timeout: Duration,
    ) -> Result<Self, XrayProcessError> {
        Self::spawn_with_binary_with_ports(
            binary_path,
            config,
            startup_timeout,
            xrat_support::readiness::RuntimeProcessPorts::default(),
        )
        .await
    }

    pub async fn spawn_with_binary_with_ports(
        binary_path: &Path,
        config: &XrayConfig,
        startup_timeout: Duration,
        ports: xrat_support::readiness::RuntimeProcessPorts,
    ) -> Result<Self, XrayProcessError> {
        let mut temp_file = tempfile::Builder::new().suffix(".json").tempfile()?;
        let config_json = serde_json::to_string_pretty(config)?;
        temp_file.write_all(config_json.as_bytes())?;
        temp_file.flush()?;

        let config_path = temp_file.path().to_path_buf();
        let local_port = config
            .inbounds
            .first()
            .and_then(|inbound| inbound.port)
            .unwrap_or(0);

        let mut command = Command::with_spawner(binary_path, ports.spawner.clone());
        configure_asset_path(&mut command, binary_path);
        let child = command
            .arg("run")
            .arg("-c")
            .arg(&config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| XrayProcessError::SpawnError(error.to_string()))?;

        let mut process = XrayProcess {
            child,
            config_file: temp_file,
            local_port,
        };
        process
            .wait_for_ready(startup_timeout, ports.waiter.as_ref())
            .await?;
        Ok(process)
    }

    async fn wait_for_ready(
        &mut self,
        timeout: Duration,
        waiter: &dyn xrat_support::readiness::PortWaiter,
    ) -> Result<(), XrayProcessError> {
        use xrat_support::readiness::{ChildPollErrorPolicy, ReadinessError, ReadinessRequest};
        let mut request = ReadinessRequest::single("127.0.0.1", self.local_port, timeout);
        request.child_errors = ChildPollErrorPolicy::Ignore;
        match waiter.wait(&mut self.child, request).await {
            Ok(()) => Ok(()),
            Err(ReadinessError::ProcessExited(_)) => {
                Err(XrayProcessError::ProcessExited(self.read_stderr()))
            }
            Err(_) => {
                let _ = self.child.kill();
                Err(XrayProcessError::PortNotReady(self.local_port))
            }
        }
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_file.path().to_path_buf()
    }

    pub fn kill(mut self) -> Result<(), std::io::Error> {
        self.child.kill()?;
        let _ = self.child.wait();
        Ok(())
    }

    pub fn wait(mut self) -> Result<std::process::ExitStatus, std::io::Error> {
        self.child.wait()
    }

    fn read_stderr(&mut self) -> String {
        let mut output = String::new();

        if let Some(stdout) = self.child.stdout.as_mut() {
            let mut buffer = String::new();
            if stdout.read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
                output.push_str(buffer.trim());
            }
        }

        if let Some(stderr) = self.child.stderr.as_mut() {
            let mut buffer = String::new();
            if stderr.read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(buffer.trim());
            }
        }

        if output.trim().is_empty() {
            "process exited without output".to_string()
        } else {
            super::diagnostics::summarize_xray_failure(&output)
        }
    }
}

fn configure_asset_path(command: &mut Command, binary_path: &Path) {
    let Some(directory) = xrat_support::platform::managed_core_asset_dir(binary_path) else {
        return;
    };
    let variable = if binary_path.file_name().and_then(|name| name.to_str()) == Some("v2ray") {
        "V2RAY_LOCATION_ASSET"
    } else {
        "XRAY_LOCATION_ASSET"
    };
    command.env(variable, directory);
}

impl Drop for XrayProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
