use super::model::{PacEndpoints, PacRules};

/// Render a deterministic PAC file. Local and private destinations bypass the
/// proxy; everything else prefers SOCKS, then HTTP. With no active proxy the
/// file routes everything `DIRECT`.
pub fn render_pac(endpoints: &PacEndpoints, rules: &PacRules) -> String {
    let mut chain: Vec<String> = Vec::new();
    if let Some((host, port)) = &endpoints.socks {
        chain.push(format!("SOCKS5 {}:{port}", pac_proxy_host(host)));
    }
    if let Some((host, port)) = &endpoints.http {
        chain.push(format!("PROXY {}:{port}", pac_proxy_host(host)));
    }
    if chain.is_empty() {
        chain.push("DIRECT".to_string());
    }
    let proxy_chain = chain.join("; ");

    if !rules.is_empty() {
        return render_pac_with_rules(&proxy_chain, rules);
    }

    format!(
        "function FindProxyForURL(url, host) {{\n\
\x20 host = host.toLowerCase();\n\
\x20 var ip = dnsResolve(host);\n\
\x20 if (\n\
\x20   isPlainHostName(host) ||\n\
\x20   shExpMatch(host, \"*.local\") ||\n\
\x20   host == \"localhost\" ||\n\
\x20   host == \"127.0.0.1\" ||\n\
\x20   host == \"::1\" ||\n\
\x20   (ip && (\n\
\x20     isInNet(ip, \"10.0.0.0\", \"255.0.0.0\") ||\n\
\x20     isInNet(ip, \"172.16.0.0\", \"255.240.0.0\") ||\n\
\x20     isInNet(ip, \"192.168.0.0\", \"255.255.0.0\")\n\
\x20   ))\n\
\x20 ) {{\n\
\x20   return \"DIRECT\";\n\
\x20 }}\n\
\x20 return \"{}\";\n\
}}\n",
        escape_js(&proxy_chain)
    )
}

fn render_pac_with_rules(proxy_chain: &str, rules: &PacRules) -> String {
    let mut pac = String::from(
        "function FindProxyForURL(url, host) {\n\
  host = host.toLowerCase();\n\
  var ip = dnsResolve(host);\n\
  if (\n\
    isPlainHostName(host) ||\n\
    shExpMatch(host, \"*.local\") ||\n\
    host == \"localhost\" ||\n\
    host == \"127.0.0.1\" ||\n\
    host == \"::1\" ||\n\
    (ip && (\n\
      isInNet(ip, \"10.0.0.0\", \"255.0.0.0\") ||\n\
      isInNet(ip, \"172.16.0.0\", \"255.240.0.0\") ||\n\
      isInNet(ip, \"192.168.0.0\", \"255.255.0.0\")\n\
    ))\n\
  ) {\n\
    return \"DIRECT\";\n\
  }\n",
    );

    append_rule_block(
        &mut pac,
        &rules.direct_domains,
        &rules.direct_cidrs,
        "DIRECT",
    );
    append_rule_block(
        &mut pac,
        &rules.block_domains,
        &rules.block_cidrs,
        "PROXY 127.0.0.1:9",
    );
    pac.push_str(&format!("  return \"{}\";\n}}\n", escape_js(proxy_chain)));
    pac
}

fn append_rule_block(pac: &mut String, domains: &[String], cidrs: &[String], action: &str) {
    let mut conditions = Vec::new();
    for domain in domains {
        if let Some(condition) = domain_condition(domain) {
            conditions.push(condition);
        }
    }
    for cidr in cidrs {
        if let Some(condition) = cidr_condition(cidr) {
            conditions.push(condition);
        }
    }
    if conditions.is_empty() {
        return;
    }

    pac.push_str("  if (\n");
    for (index, condition) in conditions.iter().enumerate() {
        let suffix = if index + 1 == conditions.len() {
            "\n"
        } else {
            " ||\n"
        };
        pac.push_str(&format!("    {condition}{suffix}"));
    }
    pac.push_str("  ) {\n");
    pac.push_str(&format!("    return \"{}\";\n", escape_js(action)));
    pac.push_str("  }\n");
}

fn domain_condition(domain: &str) -> Option<String> {
    let domain = domain.trim();
    if domain.is_empty() || domain.starts_with("geosite:") || domain.starts_with("regexp:") {
        return None;
    }
    if let Some(domain) = domain.strip_prefix("full:") {
        let domain = domain.trim_start_matches('.');
        if domain.is_empty() {
            return None;
        }
        return Some(format!("host == \"{}\"", escape_js(domain)));
    }
    let domain = domain.strip_prefix("domain:").unwrap_or(domain);
    let domain = domain.trim_start_matches('.');
    if domain.is_empty() {
        return None;
    }

    if domain.contains('*') || domain.contains('?') {
        return Some(format!("shExpMatch(host, \"{}\")", escape_js(domain)));
    }
    Some(format!(
        "(host == \"{}\" || shExpMatch(host, \"*.{}\"))",
        escape_js(domain),
        escape_js(domain)
    ))
}

fn cidr_condition(cidr: &str) -> Option<String> {
    let (ip, prefix) = cidr.trim().split_once('/')?;
    let ip: std::net::Ipv4Addr = ip.parse().ok()?;
    let prefix: u32 = prefix.parse().ok()?;
    if prefix > 32 {
        return None;
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = u32::from(ip) & mask;
    Some(format!(
        "ip && isInNet(ip, \"{}\", \"{}\")",
        std::net::Ipv4Addr::from(network),
        std::net::Ipv4Addr::from(mask)
    ))
}

fn pac_proxy_host(host: &str) -> &str {
    if host == "0.0.0.0" || host.is_empty() {
        "127.0.0.1"
    } else {
        host
    }
}

fn escape_js(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
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
}
