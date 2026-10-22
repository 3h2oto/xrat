use std::path::Path;

use crate::app::{AppError, Result};

#[cfg(windows)]
pub(super) const NAME: &str = "XRAT-Machine-API";
pub(super) const URI: &str = "urn:xrat:machine-api:v1";
pub(super) const ARGUMENTS: &str = "--profile machine serve --host 127.0.0.1";

pub(super) fn definition(executable: &Path, operator: &str) -> Result<String> {
    crate::app::services::machine_service::validate_operator(operator)?;
    if !executable.is_absolute() {
        return Err(AppError::InvalidArgument(
            "API login task requires an absolute protected executable path".into(),
        ));
    }
    let command = xml_path(executable)?;
    let directory =
        xml_path(executable.parent().ok_or_else(|| {
            AppError::InvalidArgument("API task executable has no parent".into())
        })?)?;
    let descriptor = descriptor(operator);
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><URI>{URI}</URI><Description>XRAT machine API frontend for the registered interactive operator.</Description><SecurityDescriptor>{descriptor}</SecurityDescriptor></RegistrationInfo>
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{operator}</UserId><Delay>PT20S</Delay></LogonTrigger></Triggers>
  <Principals><Principal id="Operator"><UserId>{operator}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>
  <Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries><StopIfGoingOnBatteries>false</StopIfGoingOnBatteries><AllowStartOnDemand>true</AllowStartOnDemand><Enabled>true</Enabled><ExecutionTimeLimit>PT0S</ExecutionTimeLimit><RestartOnFailure><Interval>PT1M</Interval><Count>3</Count></RestartOnFailure></Settings>
  <Actions Context="Operator"><Exec><Command>{command}</Command><Arguments>{ARGUMENTS}</Arguments><WorkingDirectory>{directory}</WorkingDirectory></Exec></Actions>
</Task>"#
    ))
}

pub(super) fn descriptor(operator: &str) -> String {
    format!("O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFX;;;{operator})")
}

fn xml_path(path: &Path) -> Result<String> {
    let path = path
        .to_str()
        .ok_or_else(|| AppError::InvalidArgument("API task paths must be valid Unicode".into()))?;
    if path.chars().any(|character| character.is_control()) {
        return Err(AppError::InvalidArgument(
            "API task paths contain control characters".into(),
        ));
    }
    Ok(path
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;"))
}
