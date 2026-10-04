use std::time::{SystemTime, UNIX_EPOCH};

/// Time source used by application services.
///
/// Services depend on this port instead of calling `SystemTime::now` or
/// `chrono`/`time` helpers so tests can advance time deterministically.
pub trait Clock: Send + Sync {
    /// Seconds since the Unix epoch.
    fn now_unix_secs(&self) -> i64;
}

/// Production clock backed by the system time.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_secs(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0)
    }
}
