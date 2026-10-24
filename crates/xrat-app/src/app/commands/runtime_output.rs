use std::net::IpAddr;
use xrat_support::net::{LocalIpResolver, SystemLocalIpResolver, format_host_port};

pub fn format_inbound_endpoint(host: &str, port: u16) -> String {
    format_inbound_endpoint_with_resolver(host, port, &SystemLocalIpResolver)
}

fn format_inbound_endpoint_with_ip<F>(host: &str, port: u16, ip_lookup: F) -> String
where
    F: FnOnce() -> Option<IpAddr>,
{
    if host == "::" {
        return format_host_port("::1", port);
    }
    if host != "0.0.0.0" {
        return format_host_port(host, port);
    }

    // A wildcard bind is not a useful copy/paste address, so substitute the
    // reachable host IP directly into the endpoint when it can be resolved.
    ip_lookup()
        .map(|ip| format_host_port(&ip.to_string(), port))
        .unwrap_or_else(|| format!("{host}:{port}"))
}

pub fn format_inbound_endpoint_with_resolver(
    host: &str,
    port: u16,
    resolver: &dyn LocalIpResolver,
) -> String {
    format_inbound_endpoint_with_ip(host, port, || resolver.primary_ip())
}

#[cfg(test)]
mod tests {
    use super::format_inbound_endpoint_with_ip;

    #[test]
    fn formats_ipv6_literal_and_wildcard_without_ipv4_resolver_substitution() {
        assert_eq!(
            format_inbound_endpoint_with_ip("::", 1080, || panic!(
                "must not substitute an IPv4 resolver address"
            )),
            "[::1]:1080"
        );
        assert_eq!(
            format_inbound_endpoint_with_ip("2001:db8::1", 1080, || None),
            "[2001:db8::1]:1080"
        );
    }

    #[test]
    fn substitutes_host_ip_for_wildcard_bind() {
        let endpoint = format_inbound_endpoint_with_ip("0.0.0.0", 1080, || {
            Some("192.0.2.10".parse().expect("test ip should parse"))
        });

        assert_eq!(endpoint, "192.0.2.10:1080");
    }

    #[test]
    fn leaves_specific_bind_unchanged() {
        let endpoint = format_inbound_endpoint_with_ip("127.0.0.1", 1080, || {
            Some("192.0.2.10".parse().expect("test ip should parse"))
        });

        assert_eq!(endpoint, "127.0.0.1:1080");
    }

    #[test]
    fn leaves_wildcard_unchanged_when_host_ip_is_unknown() {
        let endpoint = format_inbound_endpoint_with_ip("0.0.0.0", 1080, || None);

        assert_eq!(endpoint, "0.0.0.0:1080");
    }
}
