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
mod tests;
