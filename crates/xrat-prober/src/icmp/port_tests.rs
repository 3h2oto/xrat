use super::*;
use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Output};
use std::sync::{Arc, Mutex};
use xrat_support::dns::DnsResolver;
use xrat_support::platform::{Architecture, OperatingSystem, Platform};
use xrat_support::process::{Child, CommandSpec, ProcessSpawner};

struct FixtureDns {
    answers: Vec<SocketAddr>,
    fail: bool,
}
#[async_trait]
impl DnsResolver for FixtureDns {
    async fn resolve(&self, host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
        assert_eq!(
            (host, port),
            ("fixture.invalid", 0),
            "literal addresses must bypass DNS"
        );
        if self.fail {
            Err(io::Error::new(io::ErrorKind::NotFound, "fixture missing"))
        } else {
            Ok(self.answers.clone())
        }
    }
}
#[derive(Default)]
struct PingSpawner {
    args: Mutex<Vec<Vec<std::ffi::OsString>>>,
}
#[async_trait]
impl ProcessSpawner for PingSpawner {
    fn spawn(&self, _: &CommandSpec) -> io::Result<Child> {
        panic!("ping must capture async output")
    }
    fn run(&self, _: &CommandSpec, _: bool) -> io::Result<Output> {
        panic!("ping must capture async output")
    }
    async fn output_async(&self, spec: &CommandSpec) -> io::Result<Output> {
        assert_eq!(spec.program, "ping");
        self.args.lock().unwrap().push(spec.args.clone());
        Ok(Output {
            status: ExitStatus::from_raw(0),
            stdout: b"64 bytes: time=12.5 ms".to_vec(),
            stderr: Vec::new(),
        })
    }
}
#[tokio::test]
async fn literal_and_ordered_hostname_resolution_reach_injected_ping_with_platform_flags() {
    for (os, count, timeout_flag) in [
        (OperatingSystem::Linux, "-c", "-W"),
        (OperatingSystem::Macos, "-c", "-t"),
        (OperatingSystem::Freebsd, "-c", "-t"),
        (OperatingSystem::Openbsd, "-c", "-w"),
        (OperatingSystem::Windows, "-n", "-w"),
    ] {
        let dns = FixtureDns {
            answers: vec![
                "[2001:db8::1]:0".parse().unwrap(),
                "192.0.2.1:0".parse().unwrap(),
            ],
            fail: false,
        };
        let spawner = Arc::new(PingSpawner::default());
        let platform = Platform {
            os,
            arch: Architecture::X86_64,
        };
        for (input, expected_ip) in [
            ("192.0.2.1", "192.0.2.1"),
            ("2001:db8::2", "2001:db8::2"),
            ("fixture.invalid", "2001:db8::1"),
        ] {
            let result = icmp_ping_with_ports(
                input,
                Duration::from_secs(2),
                &dns,
                spawner.clone(),
                &platform,
            )
            .await;
            assert!(result.success);
            assert_eq!(result.latency_ms, Some(13));
            let args = spawner.args.lock().unwrap();
            assert_eq!(
                args.last().unwrap(),
                &[count, "1", timeout_flag, "2", expected_ip].map(std::ffi::OsString::from)
            );
        }
    }
}
#[tokio::test]
async fn empty_and_failed_dns_never_execute_ping() {
    for fail in [false, true] {
        let dns = FixtureDns {
            answers: Vec::new(),
            fail,
        };
        let spawner = Arc::new(PingSpawner::default());
        let platform = Platform {
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
        };
        let result = icmp_ping_with_ports(
            "fixture.invalid",
            Duration::from_secs(1),
            &dns,
            spawner.clone(),
            &platform,
        )
        .await;
        assert_eq!(result.failure_kind, Some(FailureKind::Dns));
        assert!(spawner.args.lock().unwrap().is_empty());
    }
}
