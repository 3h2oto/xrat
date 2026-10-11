use std::io;

fn capture_destinations(addresses: &[String]) -> Vec<String> {
    let mut destinations = Vec::new();
    for (ipv6, roots) in [
        (
            false,
            ["1/8", "2/7", "4/6", "8/5", "16/4", "32/3", "64/2", "128/1"],
        ),
        (
            true,
            [
                "100::/8", "200::/7", "400::/6", "800::/5", "1000::/4", "2000::/3", "4000::/2",
                "8000::/1",
            ],
        ),
    ] {
        if addresses
            .iter()
            .any(|address| address.contains(':') == ipv6)
        {
            destinations.extend(roots.into_iter().map(str::to_string));
        }
    }
    destinations
}

fn rows(snapshot: &str) -> impl Iterator<Item = (&str, &str, &str)> {
    let interface_column = snapshot
        .lines()
        .find_map(|line| line.split_whitespace().position(|column| column == "Netif"))
        .unwrap_or(3);
    snapshot.lines().filter_map(move |line| {
        let mut columns = line.split_whitespace();
        let destination = columns.next()?;
        let _gateway = columns.next()?;
        let flags = columns.next()?;
        let interface = columns.nth(interface_column.checked_sub(3)?)?;
        Some((destination, interface, flags))
    })
}

fn prefix_key(destination: &str) -> Option<(u128, u8, bool)> {
    let (address, prefix) = if let Some((address, prefix)) = destination.split_once('/') {
        (address, prefix.parse::<u8>().ok()?)
    } else if !destination.contains(':') {
        (
            destination,
            u8::try_from(destination.split('.').count() * 8).ok()?,
        )
    } else {
        (destination, 128)
    };
    if address.contains(':') {
        let address: std::net::Ipv6Addr = address.parse().ok()?;
        let mask = u128::MAX
            .checked_shl(u32::from(128_u8.checked_sub(prefix)?))
            .unwrap_or(0);
        Some((u128::from(address) & mask, prefix, true))
    } else {
        let mut octets = [0_u8; 4];
        for (index, value) in address.split('.').enumerate() {
            *octets.get_mut(index)? = value.parse().ok()?;
        }
        let mask = u32::MAX
            .checked_shl(u32::from(32_u8.checked_sub(prefix)?))
            .unwrap_or(0);
        Some((u128::from(u32::from_be_bytes(octets) & mask), prefix, false))
    }
}

fn check_snapshot(snapshot: &str, destinations: &[String], owned: Option<&str>) -> io::Result<()> {
    for (destination, interface, flags) in rows(snapshot) {
        if Some(interface) != owned
            && ((interface.starts_with("utun") && destination == "default" && !flags.contains('I'))
                || prefix_key(destination).is_some_and(|key| {
                    destinations
                        .iter()
                        .any(|expected| prefix_key(expected) == Some(key))
                }))
        {
            return Err(io::Error::other(format!(
                "macOS route {destination} on {interface} conflicts with managed capture; current connection preserved"
            )));
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn snapshot() -> io::Result<String> {
    let output = crate::process::Command::new("/usr/sbin/netstat")
        .args(["-rn", "-n"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(
            "could not inspect macOS routes; refusing managed capture",
        ));
    }
    String::from_utf8(output.stdout).map_err(io::Error::other)
}

#[cfg(target_os = "macos")]
pub(super) fn preflight(addresses: &[String], owned: Option<(&str, u32)>) -> io::Result<()> {
    let verified = owned.filter(|(name, index)| {
        super::macos_links::inspect(name)
            .ok()
            .flatten()
            .is_some_and(|info| info.is_tun && info.ifindex == *index && *index > 0)
    });
    let mut protected = capture_destinations(addresses);
    protected.extend_from_slice(addresses);
    check_snapshot(&snapshot()?, &protected, verified.map(|(name, _)| name))
}

#[cfg(target_os = "macos")]
pub(super) fn ready(addresses: &[String], interface: &str) -> io::Result<bool> {
    let snapshot = snapshot()?;
    Ok(capture_destinations(addresses).iter().all(|expected| {
        rows(&snapshot).any(|(destination, name, _)| {
            prefix_key(destination) == prefix_key(expected) && name == interface
        })
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_foreign_vpn_and_capture_routes_but_allows_verified_owner() {
        let destinations = capture_destinations(&["172.19.0.1/30".into(), "fdfe::1/126".into()]);
        assert_eq!(destinations.len(), 16);
        assert!(
            check_snapshot(
                "default 192.168.1.1 UGScg en0\n192.168.1 link#4 UCS en0",
                &destinations,
                None
            )
            .is_ok()
        );
        assert!(check_snapshot("1/8 172.19.0.2 UGSc en0", &destinations, None).is_err());
        assert!(check_snapshot("1 172.19.0.2 UGSc ppp0", &destinations, None).is_err());
        assert!(check_snapshot("default 10.0.0.1 UGSc utun5", &destinations, None).is_err());
        assert!(
            check_snapshot(
                "Internet6:\nDestination Gateway Flags Netif Expire\ndefault fe80::%utun0 UGcIg utun0\n",
                &destinations,
                None
            )
            .is_ok()
        );
        assert!(check_snapshot("1/8 172.19.0.2 UGScI utun5", &destinations, None).is_err());
        assert!(check_snapshot("fe80::%utun5/64 link#8 UcI utun5", &destinations, None).is_ok());
        assert!(
            check_snapshot("1/8 172.19.0.2 UGSc utun99", &destinations, Some("utun99")).is_ok()
        );
    }
}
