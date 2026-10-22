use super::definition::{self, ARGUMENTS, NAME, URI};
use crate::app::{AppError, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use std::{
    io::{self, Read},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Serialize)]
struct Request<'a> {
    operation: &'static str,
    name: &'static str,
    uri: &'static str,
    operator: Option<&'a str>,
    command: &'a str,
    directory: &'a str,
    arguments: &'static str,
    xml: Option<String>,
    descriptor: Option<String>,
}

pub(crate) fn install(operator: &str) -> Result<()> {
    run("install", Some(operator)).map(|_| ())
}

pub(crate) fn uninstall() -> Result<()> {
    run("remove", None).map(|_| ())
}

pub(crate) fn inspect(operator: &str) -> Result<(bool, bool)> {
    let state = run("inspect", Some(operator))?;
    Ok((state & 2 != 0, state & 1 != 0))
}
pub(crate) fn pause(operator: &str) -> Result<(bool, bool)> {
    let state = run("pause", Some(operator))?;
    Ok((state & 2 != 0, state & 1 != 0))
}
pub(crate) fn resume(operator: &str, enabled: bool, running: bool) -> Result<()> {
    let operation = match (enabled, running) {
        (true, true) => "resume_run",
        (true, false) => "resume_idle",
        (false, true) => "resume_run_disabled",
        (false, false) => "resume_disabled",
    };
    run(operation, Some(operator)).map(|_| ())
}
fn run(operation: &'static str, operator: Option<&str>) -> Result<u8> {
    let executable = xrat_support::windows::program_files()?.join("XRAT-Service/xrat.exe");
    let directory = executable
        .parent()
        .expect("fixed service executable has a parent");
    let path = |path: &std::path::Path| {
        path.to_str()
            .map(str::to_owned)
            .ok_or_else(|| AppError::InvalidArgument("API task paths must be valid Unicode".into()))
    };
    let command = path(&executable)?;
    let directory = path(directory)?;
    let request = Request {
        operation,
        name: NAME,
        uri: URI,
        operator,
        command: &command,
        directory: &directory,
        arguments: ARGUMENTS,
        xml: operator
            .map(|operator| definition::definition(&executable, operator))
            .transpose()?,
        descriptor: operator.map(definition::descriptor),
    };
    let payload = STANDARD.encode(serde_json::to_vec(&request)?);
    let script = format!(
        "$request = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{payload}')) | ConvertFrom-Json\n{}",
        include_str!("task.ps1")
    );
    let encoded = STANDARD.encode(
        script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    if encoded.len() > 30_000 {
        return Err(AppError::InvalidArgument(
            "API task command exceeds Windows command-line limit".into(),
        ));
    }
    let system = xrat_support::windows::system_directory()?;
    let windows = system
        .parent()
        .ok_or_else(|| io::Error::other("system directory has no parent"))?;
    let shell = system.join("WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(shell)
        .args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded])
        .env_clear()
        .env("SystemRoot", windows)
        .env("WINDIR", windows)
        .env(
            "PSModulePath",
            system.join("WindowsPowerShell/v1.0/Modules"),
        )
        .current_dir(&system)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW)
        .spawn()?;
    let mut stderr = child.stderr.take().expect("piped task stderr");
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.by_ref().take(4096).read_to_end(&mut bytes);
        let _ = io::copy(&mut stderr, &mut io::sink());
        String::from_utf8_lossy(&bytes).into_owned()
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Err(error) => break Err(error),
            Ok(None) if started.elapsed() >= Duration::from_secs(30) => {
                break Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "API Task Scheduler operation timed out",
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    if status.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let detail = reader.join().unwrap_or_default();
    let status = status?;
    if matches!(operation, "pause" | "inspect") && matches!(status.code(), Some(16..=19)) {
        return Ok((status.code().unwrap() - 16) as u8);
    }
    if !status.success() {
        return Err(AppError::InvalidArgument(format!(
            "API login task {} failed ({status}): {}",
            request.operation,
            detail.trim()
        )));
    }
    Ok(0)
}
