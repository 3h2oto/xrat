use super::*;

fn endpoints(http: bool, socks: bool) -> ActiveEndpoints {
    ActiveEndpoints {
        http: http.then(|| ("127.0.0.1".to_string(), 18201)),
        socks: socks.then(|| ("127.0.0.1".to_string(), 18200)),
        shadowsocks: None,
    }
}

#[test]
fn prefers_http_for_http_proxy_and_socks_for_all_proxy() {
    let (http_proxy, all_proxy) = proxy_urls(&endpoints(true, true), None).expect("both active");
    assert_eq!(http_proxy, "http://127.0.0.1:18201");
    assert_eq!(all_proxy, "socks5://127.0.0.1:18200");
}

#[test]
fn falls_back_when_only_socks_active() {
    let (http_proxy, all_proxy) = proxy_urls(&endpoints(false, true), None).expect("socks active");
    assert_eq!(http_proxy, "socks5://127.0.0.1:18200");
    assert_eq!(all_proxy, "socks5://127.0.0.1:18200");
}

#[test]
fn falls_back_when_only_http_active() {
    let (http_proxy, all_proxy) = proxy_urls(&endpoints(true, false), None).expect("http active");
    assert_eq!(http_proxy, "http://127.0.0.1:18201");
    assert_eq!(all_proxy, "http://127.0.0.1:18201");
}

#[test]
fn errors_when_no_inbound_active() {
    assert!(proxy_urls(&endpoints(false, false), None).is_err());
}

#[test]
fn protocol_http_forces_http_scheme_for_both_vars() {
    let (http_proxy, all_proxy) =
        proxy_urls(&endpoints(true, true), Some(ProxyShellProtocol::Http)).expect("http active");
    assert_eq!(http_proxy, "http://127.0.0.1:18201");
    assert_eq!(all_proxy, "http://127.0.0.1:18201");
}

#[test]
fn protocol_socks5_forces_socks5_scheme() {
    let (http_proxy, all_proxy) =
        proxy_urls(&endpoints(true, true), Some(ProxyShellProtocol::Socks5)).expect("socks active");
    assert_eq!(http_proxy, "socks5://127.0.0.1:18200");
    assert_eq!(all_proxy, "socks5://127.0.0.1:18200");
}

#[test]
fn protocol_socks5h_forces_socks5h_scheme() {
    let (http_proxy, all_proxy) =
        proxy_urls(&endpoints(true, true), Some(ProxyShellProtocol::Socks5h))
            .expect("socks active");
    assert_eq!(http_proxy, "socks5h://127.0.0.1:18200");
    assert_eq!(all_proxy, "socks5h://127.0.0.1:18200");
}

#[test]
fn protocol_http_errors_without_http_inbound() {
    let error = proxy_urls(&endpoints(false, true), Some(ProxyShellProtocol::Http))
        .expect_err("explicit HTTP should require HTTP inbound");
    let message = error.to_string();
    assert!(message.contains("HTTP inbound is not active"));
    assert!(message.contains("[runtime.http].enabled"));
    assert!(message.contains("omit the protocol"));
}

#[test]
fn protocol_socks_errors_without_socks_inbound() {
    for protocol in [ProxyShellProtocol::Socks5, ProxyShellProtocol::Socks5h] {
        let error = proxy_urls(&endpoints(true, false), Some(protocol))
            .expect_err("explicit SOCKS should require SOCKS inbound");
        let message = error.to_string();
        assert!(message.contains("SOCKS inbound is not active"));
        assert!(message.contains("[runtime.socks].enabled"));
        assert!(message.contains("omit the protocol"));
    }
}

#[test]
fn enable_script_bash_exports_lower_and_upper() {
    let script = enable_script(
        ProxyShellKind::Bash,
        "http://127.0.0.1:18201",
        "socks5://127.0.0.1:18200",
    );
    assert!(script.contains("export http_proxy=\"http://127.0.0.1:18201\""));
    assert!(script.contains("export HTTP_PROXY=\"http://127.0.0.1:18201\""));
    assert!(script.contains("export all_proxy=\"socks5://127.0.0.1:18200\""));
    assert!(script.contains("export ALL_PROXY=\"socks5://127.0.0.1:18200\""));
}

#[test]
fn enable_script_fish_uses_set_gx() {
    let script = enable_script(
        ProxyShellKind::Fish,
        "http://127.0.0.1:18201",
        "socks5://127.0.0.1:18200",
    );
    assert!(script.contains("set -gx http_proxy \"http://127.0.0.1:18201\""));
    assert!(script.contains("set -gx ALL_PROXY \"socks5://127.0.0.1:18200\""));
}

#[test]
fn toggle_capture_script_saves_existing_bash_values() {
    let script = capture_script(ProxyShellKind::Bash);
    assert!(script.contains("export XRAT_PROXY_OLD_http_proxy=\"$http_proxy\""));
    assert!(script.contains("export XRAT_PROXY_HAD_http_proxy=1"));
    assert!(script.contains("export XRAT_PROXY_HAD_ALL_PROXY=0"));
}

#[test]
fn toggle_restore_script_restores_or_unsets_bash_values() {
    let script = restore_script(ProxyShellKind::Bash);
    assert!(script.contains("export http_proxy=\"$XRAT_PROXY_OLD_http_proxy\""));
    assert!(script.contains("unset http_proxy"));
    assert!(script.contains("unset XRAT_PROXY_HAD_http_proxy"));
}

#[test]
fn toggle_scripts_support_fish() {
    let capture = capture_script(ProxyShellKind::Fish);
    let restore = restore_script(ProxyShellKind::Fish);
    assert!(capture.contains("set -gx XRAT_PROXY_OLD_http_proxy \"$http_proxy\""));
    assert!(restore.contains("set -gx http_proxy \"$XRAT_PROXY_OLD_http_proxy\""));
}

#[test]
fn disable_script_unsets_for_bash_and_clears_for_fish() {
    let bash = disable_script(ProxyShellKind::Zsh);
    assert!(bash.contains("unset http_proxy"));
    assert!(bash.contains("unset ALL_PROXY"));
    let fish = disable_script(ProxyShellKind::Fish);
    assert!(fish.contains("set -e http_proxy"));
    assert!(fish.contains("set -e ALL_PROXY"));
}

#[test]
fn override_takes_priority() {
    assert_eq!(
        detect_shell(Some(ProxyShellKind::Fish)),
        ProxyShellKind::Fish
    );
}

#[test]
fn usage_hint_bash_uses_eval() {
    let hint = usage_hint(ProxyShellKind::Bash, "enable", false);
    assert!(hint.starts_with("# apply: eval \"$(xrat proxy shell enable)\""));
    let hint = usage_hint(ProxyShellKind::Zsh, "disable", false);
    assert!(hint.starts_with("# apply: eval \"$(xrat proxy shell disable)\""));
}

#[test]
fn usage_hint_fish_uses_source_pipe() {
    let hint = usage_hint(ProxyShellKind::Fish, "toggle", false);
    assert!(hint.starts_with("# apply: xrat proxy shell toggle | source"));
}

#[test]
fn enable_output_prefixes_usage_hint() {
    let out = enable_output(
        ProxyShellKind::Bash,
        "http://127.0.0.1:18201",
        "socks5://127.0.0.1:18200",
        false,
    );
    assert!(out.starts_with("# apply: eval \"$(xrat proxy shell enable)\"\n"));
    assert!(out.contains("export http_proxy=\"http://127.0.0.1:18201\""));
}

#[test]
fn disable_output_prefixes_usage_hint() {
    let out = disable_output(ProxyShellKind::Fish, false);
    assert!(out.starts_with("# apply: xrat proxy shell disable | source\n"));
    assert!(out.contains("set -e http_proxy"));
}

#[test]
fn toggle_output_prefixes_usage_hint_for_both_branches() {
    let on = toggle_on_output(
        ProxyShellKind::Bash,
        "http://127.0.0.1:18201",
        "socks5://127.0.0.1:18200",
        false,
    );
    assert!(on.starts_with("# apply: eval \"$(xrat proxy shell toggle)\"\n"));
    assert!(on.contains("export XRAT_PROXY_HAD_http_proxy"));
    assert!(on.contains("export http_proxy=\"http://127.0.0.1:18201\""));

    let off = toggle_off_output(ProxyShellKind::Bash, false);
    assert!(off.starts_with("# apply: eval \"$(xrat proxy shell toggle)\"\n"));
    assert!(off.contains("unset XRAT_PROXY_HAD_http_proxy"));
}

#[test]
fn status_text_reports_post_enable_state() {
    let status = status_text_for(
        &endpoints(true, true),
        Some("socks5://127.0.0.1:18200".to_string()),
        false,
    );
    assert!(status.contains("Proxy shell"));
    assert!(status.contains("shell points at active xrat endpoints"));
    assert!(status.contains("socks5://127.0.0.1:18200"));
}

#[test]
fn status_text_reports_post_disable_state() {
    let status = status_text_for(&endpoints(true, true), None, false);
    assert!(status.contains("shell has no proxy environment set"));
    assert!(status.contains("http_proxy  -"));
}

#[test]
fn machine_proxy_shell_hints_preserve_explicit_profile_selection() {
    for kind in [
        ProxyShellKind::Bash,
        ProxyShellKind::Zsh,
        ProxyShellKind::Fish,
        ProxyShellKind::Powershell,
    ] {
        assert!(
            usage_hint(kind, "disable", true)
                .contains("xrat --profile machine proxy shell disable")
        );
    }
}

#[tokio::test]
async fn machine_shell_disable_emits_recovery_without_querying_the_owner() {
    use crate::app::daemon::ipc::DaemonResponse;
    use crate::app::services::machine_service::{
        GatewayPayload, GatewayRequest, MachineClient, Transport,
    };
    struct NoOwner;
    #[async_trait::async_trait]
    impl Transport for NoOwner {
        async fn exchange(
            &self,
            _: GatewayRequest,
        ) -> crate::app::Result<DaemonResponse<GatewayPayload>> {
            panic!("local shell recovery must not query the runtime owner");
        }
    }
    use clap::Parser;
    let service =
        ProxyService::for_machine_test(MachineClient::with_transport(std::sync::Arc::new(NoOwner)));
    let parsed = crate::cli::Cli::try_parse_from([
        "xrat",
        "proxy",
        "shell",
        "disable",
        "--shell",
        "powershell",
    ])
    .unwrap();
    let crate::cli::Command::Proxy(args) = parsed.command else {
        panic!("proxy args")
    };
    let crate::cli::ProxyAction::Shell(args) = args.action else {
        panic!("shell args")
    };
    run(&service, &args.action).await.unwrap();
}

#[test]
fn ipv6_proxy_urls_and_status_use_loopback_and_bracketed_authorities() {
    let active = ActiveEndpoints {
        http: Some(("::".into(), 18201)),
        socks: Some(("::".into(), 18200)),
        shadowsocks: None,
    };
    let urls = proxy_urls(&active, None).unwrap();
    assert_eq!(
        urls,
        ("http://[::1]:18201".into(), "socks5://[::1]:18200".into())
    );
    assert!(points_at_active(&urls.0, &active_hostports(&active)));
    assert_eq!(
        proxy_urls(&active, Some(ProxyShellProtocol::Socks5h))
            .unwrap()
            .0,
        "socks5h://[::1]:18200"
    );
}
