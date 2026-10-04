use crate::process::{Command, ProcessSpawner, Stdio, SystemProcessSpawner};
use async_trait::async_trait;
use std::sync::Arc;

#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait]
pub trait ShutdownSignal: Send + Sync {
    async fn wait(&self) -> std::io::Result<()>;
}
pub struct CtrlCShutdown;
#[async_trait]
impl ShutdownSignal for CtrlCShutdown {
    async fn wait(&self) -> std::io::Result<()> {
        tokio::signal::ctrl_c().await
    }
}
pub async fn wait_for_shutdown() -> std::io::Result<()> {
    CtrlCShutdown.wait().await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    Check,
    Default,
    Term,
    Kill,
}
pub trait ProcessSignals: Send + Sync {
    fn send(&self, pid: i64, signal: ProcessSignal) -> std::io::Result<bool>;
    fn is_running(&self, pid: i64) -> bool {
        self.send(pid, ProcessSignal::Check).unwrap_or(false)
    }
}
pub struct SystemProcessSignals {
    spawner: Arc<dyn ProcessSpawner>,
}
impl Default for SystemProcessSignals {
    fn default() -> Self {
        Self::new(Arc::new(SystemProcessSpawner))
    }
}
impl SystemProcessSignals {
    pub fn new(spawner: Arc<dyn ProcessSpawner>) -> Self {
        Self { spawner }
    }
}
impl ProcessSignals for SystemProcessSignals {
    fn send(&self, pid: i64, signal: ProcessSignal) -> std::io::Result<bool> {
        if pid <= 0 {
            return Ok(false);
        }
        #[cfg(unix)]
        {
            let mut command = Command::with_spawner("kill", self.spawner.clone());
            match signal {
                ProcessSignal::Check => {
                    command.arg("-0");
                }
                ProcessSignal::Term => {
                    command.arg("-TERM");
                }
                ProcessSignal::Kill => {
                    command.arg("-KILL");
                }
                ProcessSignal::Default => {}
            }
            command.arg(pid.to_string());
            if signal != ProcessSignal::Default {
                command.stdout(Stdio::null()).stderr(Stdio::null());
            }
            Ok(command.status()?.success())
        }
        #[cfg(not(unix))]
        {
            let _ = (signal, &self.spawner);
            Ok(false)
        }
    }
}
