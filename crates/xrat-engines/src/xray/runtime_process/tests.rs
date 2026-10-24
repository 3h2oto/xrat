use std::time::Duration;
use xrat_support::process::{Child, Command, Stdio};

use super::{TerminationOutcome, process_is_running, terminate_process_gracefully};

/// Program used to spawn a helper process that stays alive for a few seconds.
///
/// Unix ships `sleep`, Windows does not. `ping` against the loopback address
/// stands in there: it prints one reply per second, so it lives about as long
/// as `sleep 5` and can be terminated the same way.
#[cfg(unix)]
const IDLE_PROGRAM: &str = "sleep";
#[cfg(unix)]
const IDLE_ARGS: &[&str] = &["5"];

#[cfg(not(unix))]
const IDLE_PROGRAM: &str = "ping";
#[cfg(not(unix))]
const IDLE_ARGS: &[&str] = &["-n", "6", "127.0.0.1"];

/// Spawns the long-lived helper process that termination tests operate on.
fn spawn_idle_process() -> Child {
    let mut command = Command::new(IDLE_PROGRAM);
    command.args(IDLE_ARGS);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("idle helper process should spawn")
}

#[test]
fn invalid_pid_is_not_running() {
    assert!(!process_is_running(0));
    assert!(!process_is_running(-1));
}

#[test]
fn graceful_termination_ignores_invalid_pid() {
    let outcome = terminate_process_gracefully(0, Duration::from_millis(1))
        .expect("invalid pid should not fail");

    assert_eq!(outcome, TerminationOutcome::NotRunning);
}

#[test]
fn graceful_termination_stops_running_process() {
    let mut child = spawn_idle_process();

    let pid = i64::from(child.id());
    assert!(process_is_running(pid));

    let outcome = terminate_process_gracefully(pid, Duration::from_secs(1))
        .expect("termination should succeed");

    let _ = child.wait();
    assert!(matches!(
        outcome,
        TerminationOutcome::Terminated | TerminationOutcome::Killed
    ));
    assert!(!process_is_running(pid));
}
