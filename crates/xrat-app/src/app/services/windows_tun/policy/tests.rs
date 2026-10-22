use super::*;
use xrat_support::net::{NetworkAdapter, NetworkRoute};
fn fixture() -> (TunSettings, NetworkSnapshot) {
    let tun = TunSettings {
        enabled: true,
        strict_route: true,
        address: vec!["172.19.0.1/30".into(), "fd00::1/126".into()],
        ..Default::default()
    };
    let adapter = NetworkAdapter {
        name: tun.interface_name.clone(),
        identity: "owned@0001".into(),
        luid: 1,
        index: 42,
        physical: false,
        addresses: tun.address.clone(),
        dns: dns_addresses(&tun.address).unwrap(),
    };
    let routes = ["0.0.0.0/1", "128.0.0.0/1", "::/1", "8000::/1"]
        .map(|cidr| {
            let (ip, prefix) = cidr.split_once('/').unwrap();
            NetworkRoute {
                luid: 1,
                address: ip.parse().unwrap(),
                prefix: prefix.parse().unwrap(),
            }
        })
        .to_vec();
    (
        tun,
        NetworkSnapshot {
            adapters: vec![adapter],
            routes,
        },
    )
}
#[test]
fn readiness_requires_exact_owner_addresses_dns_and_complete_routes() {
    let (tun, mut snapshot) = fixture();
    snapshot.adapters[0].addresses[1] = "fd00:0:0:0:0:0:0:1/126".into();
    assert!(ready(&snapshot, &tun, "owned@0001").unwrap());
    assert!(!ready(&snapshot, &tun, "foreign").unwrap());
    snapshot.adapters[0].dns.clear();
    assert!(!ready(&snapshot, &tun, "owned@0001").unwrap());
    snapshot.adapters[0].dns = dns_addresses(&tun.address).unwrap();
    snapshot.routes.pop();
    assert!(!ready(&snapshot, &tun, "owned@0001").unwrap());
    let mut excluded = tun.clone();
    excluded.route_exclude_address = vec!["8000::/1".into()];
    assert!(ready(&snapshot, &excluded, "owned@0001").unwrap());
    snapshot.adapters[0].addresses.pop();
    assert!(!ready(&snapshot, &excluded, "owned@0001").unwrap());
}
#[test]
fn preflight_preserves_foreign_vpn_and_address_ranges_but_allows_owned_capture() {
    let (tun, mut snapshot) = fixture();
    assert!(preflight(&snapshot, &tun, Some("owned@0001")).is_ok());
    assert!(
        preflight(&snapshot, &tun, None)
            .unwrap_err()
            .contains("VPN route")
    );
    snapshot.routes.clear();
    assert!(
        preflight(&snapshot, &tun, None)
            .unwrap_err()
            .contains("overlaps")
    );
    snapshot.adapters[0].addresses = vec!["192.0.2.5/24".into()];
    assert!(preflight(&snapshot, &tun, None).is_ok());
}
#[test]
fn dns_is_one_usable_address_per_family_and_invalid_ranges_fail() {
    assert_eq!(
        dns_addresses(&[
            "172.19.0.1/30".into(),
            "fd00::1/126".into(),
            "192.0.2.1/24".into()
        ])
        .unwrap(),
        vec![
            "172.19.0.2".parse::<IpAddr>().unwrap(),
            "fd00::2".parse().unwrap()
        ]
    );
    for cidr in [
        "172.19.0.1/32",
        "172.19.0.2/30",
        "fd00::3/126",
        "::/0",
        "127.0.0.1/8",
        "bad",
    ] {
        assert!(dns_addresses(&[cidr.into()]).is_err(), "{cidr}");
    }
}
