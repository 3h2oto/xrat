use xrat_web::types::Job;

pub(crate) fn is_active(job: &Job) -> bool {
    matches!(job.state.as_str(), "queued" | "running")
}

pub(crate) fn scan_input(input: &str, port: &str) -> Result<(Vec<String>, u16), String> {
    let port = port
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| "Enter a port between 1 and 65,535.".to_string())?;
    let targets = input
        .split([',', '\n'])
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .collect::<Vec<_>>();
    if targets.is_empty() || targets.len() > 4096 {
        return Err("Enter between 1 and 4,096 IP addresses.".into());
    }
    let ips = targets.into_iter().map(|target| target.parse::<std::net::IpAddr>().map(|ip| ip.to_string()).map_err(|_| format!("{target} is not a valid IP address. Hostnames are not supported by the TCP scanner."))).collect::<Result<Vec<_>, _>>()?;
    Ok((ips, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_accepts_mixed_ip_families_and_separators() {
        let (ips, port) = scan_input(" 1.1.1.1,\n2606:4700:4700::1111\n", "443").unwrap();
        assert_eq!(ips, ["1.1.1.1", "2606:4700:4700::1111"]);
        assert_eq!(port, 443);
    }

    #[test]
    fn scan_rejects_hostnames_and_invalid_ports_or_target_counts() {
        assert!(scan_input("example.com", "443").is_err());
        for port in ["0", "65536", "", "-1"] {
            assert!(scan_input("1.1.1.1", port).is_err());
        }
        assert!(scan_input(",\n", "443").is_err());
        assert!(scan_input(&"1.1.1.1\n".repeat(4097), "443").is_err());
    }
}
