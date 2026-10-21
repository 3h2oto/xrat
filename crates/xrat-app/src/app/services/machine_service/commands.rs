use super::{installation, scm};
use crate::app::commands::output;
use crate::app::{AppError, Result};
use crate::cli::{Command, DaemonAction};
use windows_sys::Win32::System::Services::*;

pub async fn run(command: &Command) -> Result<()> {
    let Command::Daemon(daemon) = command else {
        return Err(AppError::InvalidArgument("this operation is not yet available through the machine service; the user profile was not opened".into()));
    };
    match &daemon.action {
        DaemonAction::Install(args) => return installation::install(args, false),
        DaemonAction::Uninstall(args) => return installation::uninstall(args),
        DaemonAction::RunService => return super::run_service(),
        DaemonAction::RunServer(_) => {
            return Err(AppError::InvalidArgument(
                "machine service must run through SCM; use --profile machine daemon start".into(),
            ));
        }
        DaemonAction::Start(_) => scm::start().map_err(control_error)?,
        DaemonAction::Stop(_) => scm::stop().map_err(control_error)?,
        DaemonAction::Restart(_) => scm::restart().map_err(control_error)?,
        DaemonAction::Status(_) => {}
    }
    let status = scm::status().map_err(control_error)?;
    let rows = [
        ("service", scm::SERVICE_NAME.to_string()),
        ("profile", "machine".to_string()),
        (
            "state",
            status
                .as_ref()
                .map(|status| state_name(status.dwCurrentState))
                .unwrap_or("not installed")
                .to_string(),
        ),
        (
            "pid",
            status
                .map(|status| status.dwProcessId)
                .filter(|pid| *pid > 0)
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| "-".into()),
        ),
    ];
    println!(
        "{}",
        output::format_kv(Some("Windows service"), &rows, output::color_enabled())
    );
    Ok(())
}

fn state_name(state: u32) -> &'static str {
    match state {
        SERVICE_STOPPED => "stopped",
        SERVICE_START_PENDING => "starting",
        SERVICE_STOP_PENDING => "stopping",
        SERVICE_RUNNING => "running",
        SERVICE_CONTINUE_PENDING => "resuming",
        SERVICE_PAUSE_PENDING => "pausing",
        SERVICE_PAUSED => "paused",
        _ => "unknown",
    }
}

fn control_error(error: std::io::Error) -> AppError {
    AppError::InvalidArgument(format!(
        "Windows machine service control failed: {error}; verify the service is installed and this account is its registered operator or an administrator"
    ))
}
