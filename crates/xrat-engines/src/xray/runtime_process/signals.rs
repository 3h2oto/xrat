use std::time::{Duration, Instant};

use thiserror::Error;

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Error)]
pub enum XraySignalError {
    #[error("failed to signal process: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminationOutcome {
    NotRunning,
    Terminated,
    Killed,
}

pub fn terminate_process(pid: i64) -> Result<bool, XraySignalError> {
    Ok(xrat_support::signals::ProcessSignals::send(
        &xrat_support::signals::SystemProcessSignals::default(),
        pid,
        xrat_support::signals::ProcessSignal::Default,
    )?)
}

pub fn terminate_process_gracefully(
    pid: i64,
    timeout: Duration,
) -> Result<TerminationOutcome, XraySignalError> {
    terminate_process_gracefully_with_signals(
        pid,
        timeout,
        &xrat_support::signals::SystemProcessSignals::default(),
    )
}

pub fn terminate_process_gracefully_with_signals(
    pid: i64,
    timeout: Duration,
    signals: &dyn xrat_support::signals::ProcessSignals,
) -> Result<TerminationOutcome, XraySignalError> {
    if !signals.is_running(pid) {
        return Ok(TerminationOutcome::NotRunning);
    }

    if !signals.send(pid, xrat_support::signals::ProcessSignal::Term)? {
        return Ok(TerminationOutcome::NotRunning);
    }

    let start = Instant::now();
    while start.elapsed() < timeout {
        if !signals.is_running(pid) {
            return Ok(TerminationOutcome::Terminated);
        }
        std::thread::sleep(PROCESS_POLL_INTERVAL);
    }

    if signals.is_running(pid) {
        let _ = signals.send(pid, xrat_support::signals::ProcessSignal::Kill)?;
        return Ok(TerminationOutcome::Killed);
    }

    Ok(TerminationOutcome::Terminated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use xrat_support::signals::{ProcessSignal, ProcessSignals};

    struct FakeSignals {
        running: Mutex<bool>,
        ignore_term: bool,
        fail_term: bool,
        sent: Mutex<Vec<ProcessSignal>>,
    }

    impl ProcessSignals for FakeSignals {
        fn send(&self, pid: i64, signal: ProcessSignal) -> std::io::Result<bool> {
            assert_eq!(pid, 42);
            self.sent.lock().unwrap().push(signal);
            if signal == ProcessSignal::Term && self.fail_term {
                return Err(std::io::Error::other("signal failed"));
            }
            let mut running = self.running.lock().unwrap();
            let was_running = *running;
            if signal == ProcessSignal::Kill || (signal == ProcessSignal::Term && !self.ignore_term)
            {
                *running = false;
            }
            Ok(was_running)
        }
    }

    #[test]
    fn injected_signals_cover_graceful_exit_escalation_and_absent_process() {
        for (running, ignore_term, outcome) in [
            (false, false, TerminationOutcome::NotRunning),
            (true, false, TerminationOutcome::Terminated),
            (true, true, TerminationOutcome::Killed),
        ] {
            let signals = FakeSignals {
                running: Mutex::new(running),
                ignore_term,
                fail_term: false,
                sent: Mutex::new(Vec::new()),
            };
            assert_eq!(
                terminate_process_gracefully_with_signals(42, Duration::ZERO, &signals).unwrap(),
                outcome
            );
            assert_eq!(
                signals.sent.lock().unwrap().contains(&ProcessSignal::Kill),
                outcome == TerminationOutcome::Killed
            );
        }
    }

    #[test]
    fn injected_signal_failure_is_reported() {
        let signals = FakeSignals {
            running: Mutex::new(true),
            ignore_term: false,
            fail_term: true,
            sent: Mutex::new(Vec::new()),
        };
        assert!(matches!(
            terminate_process_gracefully_with_signals(42, Duration::ZERO, &signals),
            Err(XraySignalError::Io(_))
        ));
    }
}
