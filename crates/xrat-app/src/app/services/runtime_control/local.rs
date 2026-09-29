use crate::app::Result;
use crate::app::context::AppContext;
use crate::app::daemon::ipc::RotationTrigger;
use crate::app::runtime_service::{ConnectRequest, ReplaceRequest, RuntimeService};

use super::control::{RuntimeConnectOutcome, RuntimeControl, RuntimeReplaceOutcome};

/// Drives runtime operations in the current process via [`RuntimeService`].
#[derive(Clone)]
pub struct LocalRuntimeControl {
    context: AppContext,
}

impl LocalRuntimeControl {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }
}

#[async_trait::async_trait]
impl RuntimeControl for LocalRuntimeControl {
    async fn connect(&self, config_id: i64) -> Result<RuntimeConnectOutcome> {
        let result = RuntimeService::new(&self.context)
            .connect(ConnectRequest { config_id })
            .await?;
        Ok(RuntimeConnectOutcome {
            config_id: result.config.id,
            session_id: result.session_id,
            pid: result.pid,
        })
    }

    async fn disconnect(&self) -> Result<bool> {
        let result = RuntimeService::new(&self.context).disconnect().await?;
        Ok(result.stopped_session)
    }

    async fn replace(
        &self,
        trigger: RotationTrigger,
        candidate_id: Option<i64>,
    ) -> Result<RuntimeReplaceOutcome> {
        let result = RuntimeService::new(&self.context)
            .replace(ReplaceRequest {
                trigger,
                candidate_id,
            })
            .await?;
        Ok(RuntimeReplaceOutcome {
            trigger,
            replaced: true,
            old_session_id: result.old_session_id,
            new_config_id: result.new_config_id,
            new_session_id: result.new_session_id,
            new_pid: result.new_pid,
        })
    }
}
