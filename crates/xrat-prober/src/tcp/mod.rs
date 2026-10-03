mod check;
mod classify;
mod model;

use super::FailureKind;
pub use check::{tcp_check, tcp_check_with_ports};
pub use model::TcpResult;

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::io::{Error, ErrorKind};
    use std::net::SocketAddr;
    use std::sync::Mutex;
    use std::time::Duration;
    use xrat_support::dns::DnsResolver;
    use xrat_support::readiness::{NetworkEndpoint, TcpConnector};

    struct FakeDns(Vec<SocketAddr>);
    #[async_trait]
    impl DnsResolver for FakeDns {
        async fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>> {
            assert_eq!((host, port), ("fixture.invalid", 443));
            Ok(self.0.clone())
        }
    }
    struct FakeConnector(Mutex<Vec<String>>);
    struct FailedDns(ErrorKind);
    #[async_trait]
    impl DnsResolver for FailedDns {
        async fn resolve(&self, _host: &str, _port: u16) -> std::io::Result<Vec<SocketAddr>> {
            Err(Error::new(
                self.0,
                if self.0 == ErrorKind::TimedOut {
                    "DNS lookup timed out"
                } else {
                    "fixture DNS failure"
                },
            ))
        }
    }
    struct StalledConnector(Mutex<Vec<String>>);
    #[async_trait]
    impl TcpConnector for StalledConnector {
        async fn connect(&self, endpoint: &NetworkEndpoint) -> std::io::Result<()> {
            self.0.lock().unwrap().push(endpoint.host.clone());
            std::future::pending().await
        }
    }
    #[async_trait]
    impl TcpConnector for FakeConnector {
        async fn connect(&self, endpoint: &NetworkEndpoint) -> std::io::Result<()> {
            self.0.lock().unwrap().push(endpoint.host.clone());
            assert_eq!(endpoint.port, 443);
            if endpoint.host == "::1" {
                Err(Error::new(ErrorKind::ConnectionRefused, "refused"))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn injected_dns_preserves_address_order_and_connection_fallback() {
        let dns = FakeDns(vec![
            "[::1]:443".parse().unwrap(),
            "127.0.0.1:443".parse().unwrap(),
        ]);
        let connector = FakeConnector(Mutex::new(Vec::new()));
        let result = tcp_check_with_ports(
            "fixture.invalid",
            443,
            Duration::from_secs(1),
            &dns,
            &connector,
        )
        .await;
        assert!(result.success);
        assert_eq!(*connector.0.lock().unwrap(), vec!["::1", "127.0.0.1"]);
    }

    #[tokio::test]
    async fn empty_injected_dns_result_does_not_attempt_a_connection() {
        let connector = FakeConnector(Mutex::new(Vec::new()));
        let result = tcp_check_with_ports(
            "fixture.invalid",
            443,
            Duration::from_secs(1),
            &FakeDns(Vec::new()),
            &connector,
        )
        .await;
        assert_eq!(result.failure_kind, Some(FailureKind::Dns));
        assert!(connector.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn injected_dns_errors_preserve_classification_without_connecting() {
        for (error, expected) in [
            (ErrorKind::NotFound, FailureKind::Dns),
            (ErrorKind::TimedOut, FailureKind::Timeout),
        ] {
            let connector = FakeConnector(Mutex::new(Vec::new()));
            let result = tcp_check_with_ports(
                "fixture.invalid",
                443,
                Duration::from_secs(1),
                &FailedDns(error),
                &connector,
            )
            .await;
            assert_eq!(result.failure_kind, Some(expected));
            assert!(connector.0.lock().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn stalled_injected_connection_times_out_without_attempting_next_address() {
        let dns = FakeDns(vec![
            "[::1]:443".parse().unwrap(),
            "127.0.0.1:443".parse().unwrap(),
        ]);
        let connector = StalledConnector(Mutex::new(Vec::new()));
        let result = tcp_check_with_ports(
            "fixture.invalid",
            443,
            Duration::from_millis(1),
            &dns,
            &connector,
        )
        .await;
        assert_eq!(result.failure_kind, Some(FailureKind::Timeout));
        assert_eq!(*connector.0.lock().unwrap(), vec!["::1"]);
    }

    #[test]
    fn classifies_connection_refused_errors() {
        let (kind, reason) =
            classify::classify_tcp_error(&Error::new(ErrorKind::ConnectionRefused, "refused"));
        assert_eq!(kind, FailureKind::Refused);
        assert_eq!(reason, "Connection refused");
    }

    #[test]
    fn classifies_timeout_errors() {
        let (kind, reason) =
            classify::classify_tcp_error(&Error::new(ErrorKind::TimedOut, "timed out"));
        assert_eq!(kind, FailureKind::Timeout);
        assert_eq!(reason, "Connection timed out");
    }

    #[test]
    fn classifies_permission_errors() {
        let (kind, reason) =
            classify::classify_tcp_error(&Error::new(ErrorKind::PermissionDenied, "blocked"));
        assert_eq!(kind, FailureKind::PermissionDenied);
        assert_eq!(reason, "Permission denied");
    }

    #[tokio::test]
    async fn classifies_dns_lookup_failures() {
        let result = tcp_check("definitely-not-a-host.invalid", 80, Duration::from_secs(1)).await;
        assert!(!result.success);
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::Dns) | Some(FailureKind::Timeout)
        ));
    }
}
