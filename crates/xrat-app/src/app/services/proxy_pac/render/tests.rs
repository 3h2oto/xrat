use super::*;

#[test]
fn renders_socks_preferred_chain() {
    let pac = render_pac(
        &PacEndpoints {
            http: Some(("127.0.0.1".to_string(), 18201)),
            socks: Some(("127.0.0.1".to_string(), 18200)),
        },
        &PacRules::default(),
    );
    assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200; PROXY 127.0.0.1:18201\";"));
    assert!(pac.contains("isPlainHostName(host)"));
    assert!(pac.contains("host = host.toLowerCase();"));
    assert!(pac.contains("var ip = dnsResolve(host);"));
    assert!(pac.contains("ip && ("));
}

#[test]
fn renders_http_only_chain() {
    let pac = render_pac(
        &PacEndpoints {
            http: Some(("127.0.0.1".to_string(), 18201)),
            socks: None,
        },
        &PacRules::default(),
    );
    assert!(pac.contains("return \"PROXY 127.0.0.1:18201\";"));
}

#[test]
fn renders_socks_only_chain() {
    let pac = render_pac(
        &PacEndpoints {
            http: None,
            socks: Some(("127.0.0.1".to_string(), 18200)),
        },
        &PacRules::default(),
    );
    assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200\";"));
}

#[test]
fn renders_loopback_for_wildcard_proxy_hosts() {
    let pac = render_pac(
        &PacEndpoints {
            http: Some(("0.0.0.0".to_string(), 18201)),
            socks: Some(("0.0.0.0".to_string(), 18200)),
        },
        &PacRules::default(),
    );
    assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200; PROXY 127.0.0.1:18201\";"));
    assert!(!pac.contains("SOCKS5 0.0.0.0"));
    assert!(!pac.contains("PROXY 0.0.0.0"));
}

#[test]
fn renders_direct_when_no_proxy_active() {
    let pac = render_pac(&PacEndpoints::default(), &PacRules::default());
    assert!(pac.contains("return \"DIRECT\";"));
}

#[test]
fn renders_direct_domain_rule() {
    let pac = render_pac(
        &PacEndpoints::default(),
        &PacRules {
            direct_domains: vec!["example.com".to_string()],
            ..PacRules::default()
        },
    );

    assert!(pac.contains("host == \"example.com\""));
    assert!(pac.contains("shExpMatch(host, \"*.example.com\")"));
    assert!(pac.contains("return \"DIRECT\";"));
}

#[test]
fn renders_cidr_rule_with_mask() {
    let pac = render_pac(
        &PacEndpoints::default(),
        &PacRules {
            direct_cidrs: vec!["203.0.113.9/24".to_string()],
            ..PacRules::default()
        },
    );

    assert!(pac.contains("ip && isInNet(ip, \"203.0.113.0\", \"255.255.255.0\")"));
}

#[test]
fn renders_block_rule_before_default_proxy() {
    let pac = render_pac(
        &PacEndpoints {
            http: Some(("127.0.0.1".to_string(), 18201)),
            socks: None,
        },
        &PacRules {
            block_domains: vec!["blocked.example".to_string()],
            ..PacRules::default()
        },
    );

    let block_index = pac
        .find("blocked.example")
        .expect("block rule should render");
    let default_index = pac
        .find("return \"PROXY 127.0.0.1:18201\";")
        .expect("default proxy fallback should render");
    assert!(block_index < default_index);
    assert!(pac.contains("return \"PROXY 127.0.0.1:9\";"));
}
