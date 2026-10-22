#[cfg(any(windows, test))]
use crate::app::config::TunSettings;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
#[cfg(any(windows, test))]
use xrat_support::net::NetworkSnapshot;

fn range(cidr: &str) -> Result<(bool, u128, u128), String> {
    let (ip, prefix) = cidr.split_once('/').ok_or("missing CIDR prefix")?;
    let ip: IpAddr = ip.parse().map_err(|_| "invalid CIDR address")?;
    let prefix: u32 = prefix.parse().map_err(|_| "invalid CIDR prefix")?;
    let bits = if ip.is_ipv4() { 32 } else { 128 };
    if prefix > bits {
        return Err("invalid CIDR prefix".into());
    }
    let value = match ip {
        IpAddr::V4(ip) => u128::from(u32::from(ip)),
        IpAddr::V6(ip) => u128::from(ip),
    };
    let hosts = if bits - prefix == 128 {
        u128::MAX
    } else {
        (1u128 << (bits - prefix)) - 1
    };
    Ok((ip.is_ipv4(), value & !hosts, value | hosts))
}
pub(crate) fn dns_addresses(addresses: &[String]) -> Result<Vec<IpAddr>, String> {
    let mut output = Vec::new();
    for cidr in addresses {
        let ip: IpAddr = cidr
            .split_once('/')
            .ok_or("missing CIDR prefix")?
            .0
            .parse()
            .map_err(|_| "invalid TUN address")?;
        if output
            .iter()
            .any(|entry: &IpAddr| entry.is_ipv4() == ip.is_ipv4())
        {
            continue;
        }
        let (_, _, end) = range(cidr)?;
        let (value, next) = match ip {
            IpAddr::V4(ip) => {
                let value = u32::from(ip);
                (
                    u128::from(value),
                    IpAddr::V4(Ipv4Addr::from(
                        value.checked_add(1).ok_or("TUN DNS address overflow")?,
                    )),
                )
            }
            IpAddr::V6(ip) => {
                let value = u128::from(ip);
                (
                    value,
                    IpAddr::V6(Ipv6Addr::from(
                        value.checked_add(1).ok_or("TUN DNS address overflow")?,
                    )),
                )
            }
        };
        if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() || value + 1 >= end {
            return Err(
                "Windows TUN address must leave a usable next address for DNS inside its CIDR"
                    .into(),
            );
        }
        output.push(next);
    }
    if output.is_empty() {
        return Err("Windows TUN requires interface addresses".into());
    }
    Ok(output)
}
#[cfg(any(windows, test))]
fn covered(ipv4: bool, ranges: impl Iterator<Item = String>) -> Result<bool, String> {
    let mut ranges = ranges
        .map(|cidr| range(&cidr))
        .collect::<Result<Vec<_>, _>>()?;
    ranges.retain(|entry| entry.0 == ipv4);
    ranges.sort_unstable_by_key(|entry| entry.1);
    let maximum = if ipv4 {
        u128::from(u32::MAX)
    } else {
        u128::MAX
    };
    let mut next = 0;
    for (_, first, last) in ranges {
        if first > next {
            return Ok(false);
        }
        if last >= maximum {
            return Ok(true);
        }
        next = next.max(last + 1);
    }
    Ok(false)
}
#[cfg(any(windows, test))]
pub(crate) fn ready(
    snapshot: &NetworkSnapshot,
    tun: &TunSettings,
    identity: &str,
) -> Result<bool, String> {
    let Some(adapter) = snapshot
        .adapters
        .iter()
        .find(|adapter| adapter.name == tun.interface_name && adapter.identity == identity)
    else {
        return Ok(false);
    };
    let parse = |address: &str| -> Option<(IpAddr, u8)> {
        let (ip, prefix) = address.split_once('/')?;
        Some((ip.parse().ok()?, prefix.parse().ok()?))
    };
    if !tun.address.iter().all(|address| {
        parse(address).is_some_and(|expected| {
            adapter
                .addresses
                .iter()
                .any(|current| parse(current) == Some(expected))
        })
    }) {
        return Ok(false);
    }
    if !tun.auto_route {
        return Ok(true);
    }
    if !dns_addresses(&tun.address)?
        .iter()
        .all(|dns| adapter.dns.contains(dns))
    {
        return Ok(false);
    }
    for ipv4 in [true, false] {
        if !tun
            .address
            .iter()
            .any(|cidr| range(cidr).is_ok_and(|entry| entry.0 == ipv4))
        {
            continue;
        }
        let routes = snapshot
            .routes
            .iter()
            .filter(|route| route.luid == adapter.luid)
            .map(|route| format!("{}/{}", route.address, route.prefix));
        if !covered(
            ipv4,
            routes.chain(tun.route_exclude_address.iter().cloned()),
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}
#[cfg(any(windows, test))]
pub(crate) fn preflight(
    snapshot: &NetworkSnapshot,
    tun: &TunSettings,
    owned: Option<&str>,
) -> Result<(), String> {
    for route in &snapshot.routes {
        let adapter = snapshot
            .adapters
            .iter()
            .find(|adapter| adapter.luid == route.luid)
            .ok_or("network adapters changed during TUN preflight; retry")?;
        if owned == Some(adapter.identity.as_str()) {
            continue;
        }
        if !adapter.physical && route.prefix <= 1 && !route.address.is_loopback() {
            return Err(format!(
                "Windows capture conflicts with an unowned VPN route on {}; current connection preserved",
                adapter.name
            ));
        }
    }
    for adapter in &snapshot.adapters {
        if owned == Some(adapter.identity.as_str()) {
            continue;
        }
        for current in &adapter.addresses {
            let current = range(current)?;
            for requested in &tun.address {
                let requested = range(requested)?;
                if current.0 == requested.0 && current.1 <= requested.2 && requested.1 <= current.2
                {
                    return Err(format!(
                        "Windows TUN address range overlaps interface {}; current connection preserved",
                        adapter.name
                    ));
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests;
