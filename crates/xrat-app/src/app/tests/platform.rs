//! Cross-platform helpers for application tests.
//!
//! Unix-only APIs (executable bits, raw wait statuses, the `sleep` binary)
//! have no direct Windows equivalent, so the tests call these wrappers
//! instead of sprinkling `#[cfg(unix)]` over every call site.

use std::path::Path;
use std::process::ExitStatus;

use xrat_support::process::{Child, Command, Stdio};

/// Makes `path` executable on platforms that carry an executable bit.
///
/// Windows has no executable bit, so the call is a no-op there and the file
/// keeps the permissions it was created with.
pub(crate) fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)
            .expect("test script metadata should load")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("test script should be executable");
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// Builds an [`ExitStatus`] that reports `code` as the process exit code.
///
/// On Unix a raw wait status encodes the exit code in the high byte, which is
/// what `ExitStatus::from_raw` expects. Windows stores the exit code
/// directly, so the raw value is the code itself.
pub(crate) fn exit_status(code: i32) -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code << 8)
    }
    #[cfg(not(unix))]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }
}

/// Spawns a child process that keeps running for `seconds`.
///
/// Unix uses `sleep`. Windows has no `sleep` binary, so the helper pings the
/// loopback address instead, which keeps the process alive for the same
/// amount of time.
pub(crate) fn spawn_sleep(seconds: u64) -> Child {
    #[cfg(unix)]
    let mut command = {
        let mut command = Command::new("sleep");
        command.arg(seconds.to_string());
        command
    };
    #[cfg(not(unix))]
    let mut command = {
        let count = (seconds + 1).to_string();
        let mut command = Command::new("ping");
        command.args(["-n", count.as_str(), "127.0.0.1"]);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_executable_keeps_the_fixture_usable() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let script = dir.path().join("fixture.sh");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").expect("script should be written");

        make_executable(&script);

        assert!(script.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&script)
                .expect("script metadata should load")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }
    }

    #[test]
    fn exit_status_reports_the_requested_exit_code() {
        assert!(exit_status(0).success());
        assert_eq!(exit_status(0).code(), Some(0));
        assert!(!exit_status(1).success());
        assert_eq!(exit_status(1).code(), Some(1));
    }
}
