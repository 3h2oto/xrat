use super::*;
use crate::app::config::AppConfig;

fn endpoints() -> ActiveEndpoints {
    ActiveEndpoints {
        http: Some(("127.0.0.1".to_string(), 18201)),
        socks: Some(("127.0.0.1".to_string(), 18200)),
        shadowsocks: None,
    }
}

#[test]
fn manual_enable_sets_mode_and_hosts() {
    let commands = gnome_enable_commands(&endpoints(), None).expect("commands");
    assert_eq!(
        commands[0],
        vec!["set", "org.gnome.system.proxy", "mode", "manual"]
    );
    assert!(commands.iter().any(|c| c
        == &vec![
            "set".to_string(),
            "org.gnome.system.proxy.socks".to_string(),
            "port".to_string(),
            "18200".to_string()
        ]));
}

#[test]
fn pac_enable_uses_auto_mode() {
    let commands = gnome_enable_commands(
        &ActiveEndpoints::default(),
        Some("http://127.0.0.1:8787/proxy.pac"),
    )
    .expect("commands");
    assert_eq!(
        commands[0],
        vec!["set", "org.gnome.system.proxy", "mode", "auto"]
    );
    assert_eq!(
        commands[1],
        vec![
            "set",
            "org.gnome.system.proxy",
            "autoconfig-url",
            "http://127.0.0.1:8787/proxy.pac"
        ]
    );
}

#[test]
fn pac_url_requires_server_and_pac_enabled() {
    let mut config = AppConfig::default();
    config.server.enabled = true;
    config.server.pac_enabled = false;
    assert!(pac_url_from_server(&config.server).is_err());

    config.server.enabled = false;
    config.server.pac_enabled = true;
    assert!(pac_url_from_server(&config.server).is_err());

    config.server.enabled = true;
    assert_eq!(
        pac_url_from_server(&config.server).expect("pac url"),
        "http://127.0.0.1:18203/proxy.pac"
    );
}

#[test]
fn manual_enable_requires_an_active_inbound() {
    assert!(gnome_enable_commands(&ActiveEndpoints::default(), None).is_err());
}

#[test]
fn toggle_enables_when_mode_is_none() {
    let commands = gnome_toggle_commands(&endpoints(), None, "'none'\n").expect("commands");
    assert_eq!(
        commands[0],
        vec!["set", "org.gnome.system.proxy", "mode", "manual"]
    );
}

#[test]
fn toggle_disables_when_mode_is_manual_or_auto() {
    for mode in ["'manual'", "'auto'"] {
        assert_eq!(
            gnome_toggle_commands(&endpoints(), None, mode).expect("commands"),
            gnome_disable_commands()
        );
    }
}

#[test]
fn toggle_can_enable_with_pac() {
    let commands = gnome_toggle_commands(
        &ActiveEndpoints::default(),
        Some("http://127.0.0.1:8787/proxy.pac"),
        "none",
    )
    .expect("commands");
    assert_eq!(
        commands[0],
        vec!["set", "org.gnome.system.proxy", "mode", "auto"]
    );
}

#[test]
fn parses_enabled_network_services() {
    let output = "An asterisk (*) denotes that a network service is disabled.\nWi-Fi\nThunderbolt Bridge\n*Disabled Service\n";
    assert_eq!(
        parse_network_services(output),
        vec!["Wi-Fi".to_string(), "Thunderbolt Bridge".to_string()]
    );
}

#[test]
fn detects_desktops_from_hints() {
    assert_eq!(desktop_from_hint("gnome"), Some(ProxyDesktopKind::Gnome));
    assert_eq!(
        desktop_from_hint("ubuntu:gnome"),
        Some(ProxyDesktopKind::Gnome)
    );
    assert_eq!(desktop_from_hint("kde"), Some(ProxyDesktopKind::Kde));
    assert_eq!(desktop_from_hint("xfce"), Some(ProxyDesktopKind::Xfce));
    assert_eq!(desktop_from_hint("sway"), None);
}
