use std::net::IpAddr;
use std::time::{Duration, Instant};
use xrat_support::process::Command;

use super::FailureKind;

mod parsing;

#[cfg(all(test, unix))]
mod port_tests;
#[cfg(test)]
mod tests;

pub use parsing::{classify_ping_failure, parse_ping_latency};

mod runner;
pub use runner::IcmpResult;
pub use runner::icmp_ping;
pub use runner::icmp_ping_with_ports;
