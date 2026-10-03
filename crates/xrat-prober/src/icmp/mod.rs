use std::net::IpAddr;
use std::time::{Duration, Instant};
use xrat_support::process::Command;

use super::FailureKind;

mod parsing;

#[cfg(test)]
mod tests;

pub use parsing::{classify_ping_failure, parse_ping_latency};

#[derive(Debug, Clone)]
pub struct IcmpResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

pub async fn icmp_ping(address: &str, timeout: Duration) -> IcmpResult {
    icmp_ping_with_ports(
        address,
        timeout,
        &xrat_support::dns::TokioDnsResolver,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
        &xrat_support::platform::HostPlatformDetector,
    )
    .await
}

pub async fn icmp_ping_with_ports(
    address: &str,
    timeout: Duration,
    resolver: &dyn xrat_support::dns::DnsResolver,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
    platform: &dyn xrat_support::platform::PlatformDetector,
) -> IcmpResult {
    let ip = match resolve_address_with_resolver(address, resolver).await {
        Ok(ip) => ip,
        Err(error) => {
            return IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(FailureKind::Dns),
                failure_reason: Some(error),
            };
        }
    };

    ping_with_ports(&ip.to_string(), timeout, spawner, platform).await
}

async fn resolve_address_with_resolver(
    address: &str,
    resolver: &dyn xrat_support::dns::DnsResolver,
) -> Result<IpAddr, String> {
    if let Ok(ip) = address.parse::<IpAddr>() {
        return Ok(ip);
    }

    resolver
        .resolve(address, 0)
        .await
        .map_err(|error| format!("DNS resolution failed: {error}"))?
        .into_iter()
        .next()
        .map(|addr| addr.ip())
        .ok_or_else(|| "No IP address found".to_string())
}

async fn ping_with_ports(
    ip: &str,
    timeout: Duration,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
    platform: &dyn xrat_support::platform::PlatformDetector,
) -> IcmpResult {
    let timeout_secs = timeout.as_secs().max(1).to_string();
    let (count_flag, timeout_flag) = ping_flags_with_platform(platform);

    let start = Instant::now();
    let output = Command::with_spawner("ping", spawner)
        .arg(count_flag)
        .arg("1")
        .arg(timeout_flag)
        .arg(&timeout_secs)
        .arg(ip)
        .output_async()
        .await;

    let elapsed = start.elapsed();
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let latency = parse_ping_latency(&stdout).unwrap_or(elapsed.as_millis() as u32);
            IcmpResult {
                success: true,
                latency_ms: Some(latency),
                failure_kind: None,
                failure_reason: None,
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let (kind, reason) = classify_ping_failure(&format!("{stdout}{stderr}"));
            IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            }
        }
        Err(error) => {
            let kind = if error.kind() == std::io::ErrorKind::PermissionDenied {
                FailureKind::PermissionDenied
            } else {
                FailureKind::Unknown
            };
            IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(format!("Failed to execute ping: {error}")),
            }
        }
    }
}

fn ping_flags_with_platform(
    detector: &dyn xrat_support::platform::PlatformDetector,
) -> (&'static str, &'static str) {
    use xrat_support::platform::OperatingSystem;
    match detector.detect().os {
        OperatingSystem::Macos | OperatingSystem::Freebsd => ("-c", "-t"),
        OperatingSystem::Openbsd => ("-c", "-w"),
        OperatingSystem::Windows => ("-n", "-w"),
        _ => ("-c", "-W"),
    }
}
