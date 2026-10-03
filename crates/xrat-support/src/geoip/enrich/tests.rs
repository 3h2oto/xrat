use std::net::IpAddr;

use super::*;
use crate::geoip::GeoIpError;

#[derive(Debug)]
struct TestLookup;

struct FixtureDns {
    answers: Vec<std::net::SocketAddr>,
    fail: bool,
    calls: std::sync::atomic::AtomicUsize,
}
#[async_trait::async_trait]
impl crate::dns::DnsResolver for FixtureDns {
    async fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<std::net::SocketAddr>> {
        assert_eq!((host, port), ("fixture.invalid", 0));
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.fail {
            Err(std::io::Error::other("DNS failed"))
        } else {
            Ok(self.answers.clone())
        }
    }
}

#[tokio::test]
async fn injected_resolution_preserves_literal_bypass_and_hostname_provenance() {
    let dns = FixtureDns {
        answers: vec![
            "[2001:db8::1]:0".parse().unwrap(),
            "192.0.2.1:0".parse().unwrap(),
        ],
        fail: false,
        calls: std::sync::atomic::AtomicUsize::new(0),
    };
    for literal in ["192.0.2.1:443", "[2001:db8::1]"] {
        let meta = enrich_address_with_resolver(literal, &TestLookup, &dns).await;
        assert_eq!(meta.source, Some(GeoIpSource::LiteralIp));
    }
    assert_eq!(dns.calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    let resolved = resolve_address_ip_with_source("https://fixture.invalid:443/path", &dns)
        .await
        .unwrap();
    assert_eq!(
        resolved,
        ("2001:db8::1".parse().unwrap(), GeoIpSource::DialDns)
    );
    let meta = enrich_address_with_resolver("fixture.invalid", &TestLookup, &dns).await;
    assert_eq!(meta.source, Some(GeoIpSource::DialDns));
    assert_eq!(meta.country.as_deref(), Some("ZZ"));
}

#[tokio::test]
async fn failed_or_empty_injected_dns_does_not_create_geoip_metadata() {
    for fail in [false, true] {
        let dns = FixtureDns {
            answers: Vec::new(),
            fail,
            calls: std::sync::atomic::AtomicUsize::new(0),
        };
        assert_eq!(
            enrich_address_with_resolver("fixture.invalid", &TestLookup, &dns).await,
            EndpointGeoMeta::default()
        );
        assert_eq!(dns.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for TestLookup {
    async fn country(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(Some("ZZ".to_string()))
    }

    async fn city(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(Some("ZZ/Test City".to_string()))
    }

    async fn asn(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(Some("AS64512 TEST".to_string()))
    }

    fn backend_name(&self) -> &'static str {
        "test"
    }
}

#[tokio::test]
async fn enriches_literal_ip_addresses() {
    let meta = enrich_address("8.8.8.8", &TestLookup).await;

    assert_eq!(meta.country.as_deref(), Some("ZZ"));
    assert_eq!(meta.location.as_deref(), Some("ZZ/Test City"));
    assert_eq!(meta.asn.as_deref(), Some("AS64512 TEST"));
    assert_eq!(meta.source, Some(GeoIpSource::LiteralIp));
    assert_eq!(meta.fronting, None);
}

#[test]
fn extracts_hosts_from_lookup_inputs() {
    assert_eq!(address_host("8.8.8.8").as_deref(), Some("8.8.8.8"));
    assert_eq!(
        address_host("https://google.com:443/path").as_deref(),
        Some("google.com")
    );
    assert_eq!(
        address_host("google.com:443").as_deref(),
        Some("google.com")
    );
    assert_eq!(
        address_host("[2001:4860:4860::8888]").as_deref(),
        Some("2001:4860:4860::8888")
    );
}
