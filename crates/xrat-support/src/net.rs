pub fn connect_host_for_bind_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "::1".to_string(),
        _ => host.to_string(),
    }
}

/// Best-effort primary LAN IP of this host. Opens a UDP socket toward a public
/// address (no packets are sent) so the OS picks the outbound interface, then
/// reads back its local address. Returns `None` when no route can be resolved.
pub trait LocalIpResolver: Send + Sync {
    fn primary_ip(&self) -> Option<std::net::IpAddr>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemLocalIpResolver;
impl LocalIpResolver for SystemLocalIpResolver {
    fn primary_ip(&self) -> Option<std::net::IpAddr> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        socket.local_addr().ok().map(|addr| addr.ip())
    }
}

pub fn primary_local_ip() -> Option<String> {
    SystemLocalIpResolver.primary_ip().map(|ip| ip.to_string())
}

/// Resolve a network interface name to a bindable address, preferring IPv4.
/// Used to turn an inbound `listen_interface` setting into a concrete `listen`
/// address. Returns `None` when the interface is unknown or has no address.
pub fn interface_address(name: &str) -> Option<String> {
    let addrs = if_addrs::get_if_addrs().ok()?;
    let mut ipv6: Option<String> = None;
    for iface in addrs {
        if iface.name != name {
            continue;
        }
        let ip = iface.ip();
        if ip.is_ipv4() {
            return Some(ip.to_string());
        }
        ipv6.get_or_insert_with(|| ip.to_string());
    }
    ipv6
}

/// Whether a network interface with `name` currently exists. Checks the Linux
/// sysfs view first so interfaces without an assigned address are still
/// detected, then falls back to the cross-platform address list.
pub fn interface_exists(name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() {
        return false;
    }
    if std::path::Path::new("/sys/class/net").join(name).exists() {
        return true;
    }
    if_addrs::get_if_addrs()
        .map(|addrs| addrs.iter().any(|iface| iface.name == name))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::interface_exists;

    #[test]
    fn detects_loopback_and_missing_interfaces() {
        assert!(interface_exists("lo"));
        assert!(!interface_exists("xrat-missing-interface"));
        assert!(!interface_exists("  "));
    }
}
