use std::path::PathBuf;

use crate::app::daemon::ipc;
use crate::app::daemon::ipc::RotationTrigger;
use crate::app::{AppError, Result};
use xrat_model::ConfigId;

use super::control::{RuntimeConnectOutcome, RuntimeControl, RuntimeReplaceOutcome};

/// Drives runtime operations through a running daemon over the IPC socket.
#[derive(Clone)]
pub struct DaemonRuntimeControl {
    socket_path: PathBuf,
}

impl DaemonRuntimeControl {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    fn unreachable_hint(&self) -> AppError {
        AppError::InvalidArgument(format!(
            "daemon is not running. Start it with `xrat daemon start` (socket: {})",
            self.socket_path.display()
        ))
    }
}

#[async_trait::async_trait]
impl RuntimeControl for DaemonRuntimeControl {
    async fn connect(&self, config_id: ConfigId) -> Result<RuntimeConnectOutcome> {
        let response = match ipc::runtime_connect_daemon(&self.socket_path, config_id).await {
            Ok(response) => response,
            Err(err) if ipc::daemon_unreachable(&err) => return Err(self.unreachable_hint()),
            Err(err) => return Err(err),
        };
        if !response.ok {
            return Err(AppError::InvalidArgument(response.message));
        }
        let payload = response.payload.ok_or_else(|| {
            AppError::InvalidArgument("daemon connect response missing payload".to_string())
        })?;
        Ok(RuntimeConnectOutcome {
            config_id: payload.config_id,
            session_id: payload.session_id,
            pid: payload.pid,
        })
    }

    async fn disconnect(&self) -> Result<bool> {
        let response = match ipc::runtime_disconnect_daemon(&self.socket_path).await {
            Ok(response) => response,
            Err(err) if ipc::daemon_unreachable(&err) => return Err(self.unreachable_hint()),
            Err(err) => return Err(err),
        };
        if !response.ok {
            return Err(AppError::InvalidArgument(response.message));
        }
        let payload = response.payload.ok_or_else(|| {
            AppError::InvalidArgument("daemon disconnect response missing payload".to_string())
        })?;
        Ok(payload.stopped_session)
    }

    async fn replace(
        &self,
        trigger: RotationTrigger,
        candidate_id: Option<ConfigId>,
    ) -> Result<RuntimeReplaceOutcome> {
        let response =
            match ipc::runtime_replace_daemon(&self.socket_path, trigger, candidate_id).await {
                Ok(response) => response,
                Err(err) if ipc::daemon_unreachable(&err) => return Err(self.unreachable_hint()),
                Err(err) => return Err(err),
            };
        if !response.ok {
            return Err(AppError::InvalidArgument(response.message));
        }
        let payload = response.payload.ok_or_else(|| {
            AppError::InvalidArgument("daemon replace response missing payload".to_string())
        })?;
        Ok(RuntimeReplaceOutcome {
            trigger: payload.trigger,
            replaced: payload.replaced,
            old_session_id: payload.old_session_id,
            new_config_id: payload.new_config_id,
            new_session_id: payload.new_session_id,
            new_pid: payload.new_pid,
        })
    }
}
