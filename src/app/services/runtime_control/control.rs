use crate::app::Result;
use crate::app::daemon::ipc::RotationTrigger;

/// Result of a successful connect operation, independent of control path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConnectOutcome {
    pub config_id: i64,
    pub session_id: i64,
    pub pid: u32,
}

/// Result of a successful replace operation, independent of control path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeReplaceOutcome {
    pub trigger: RotationTrigger,
    pub replaced: bool,
    pub old_session_id: Option<i64>,
    pub new_config_id: i64,
    pub new_session_id: i64,
    pub new_pid: u32,
}

/// Runtime control operations shared by CLI, TUI, and daemon callers.
///
/// Implementations differ in where the work happens: `DaemonRuntimeControl`
/// drives a running daemon over IPC, while `LocalRuntimeControl` starts and
/// stops processes in the current process. Callers that need consistent
/// semantics depend on this trait instead of choosing a path themselves.
#[async_trait::async_trait]
pub trait RuntimeControl: Send + Sync {
    /// Connect a runtime session for the given config.
    async fn connect(&self, config_id: i64) -> Result<RuntimeConnectOutcome>;

    /// Disconnect the active runtime session. Returns whether a session stopped.
    async fn disconnect(&self) -> Result<bool>;

    /// Replace the active runtime session with a candidate config.
    async fn replace(
        &self,
        trigger: RotationTrigger,
        candidate_id: Option<i64>,
    ) -> Result<RuntimeReplaceOutcome>;
}
