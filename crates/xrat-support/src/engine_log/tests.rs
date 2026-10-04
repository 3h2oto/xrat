use super::*;

#[test]
fn parses_xray_warning_line_with_component() {
    let row = EngineLogRow::parse(
        "xray",
        ProxyStream::Stdout,
        "2026/06/06 15:10:29.760343 [Warning] core: Xray 26.3.27 started",
    );
    assert_eq!(row.time.as_deref(), Some("2026/06/06 15:10:29.760343"));
    assert_eq!(row.level.as_deref(), Some("Warning"));
    assert_eq!(row.component.as_deref(), Some("core"));
    assert_eq!(row.message, "Xray 26.3.27 started");
}

#[test]
fn parses_xray_line_without_component() {
    let row = EngineLogRow::parse(
        "xray",
        ProxyStream::Stdout,
        "2026/06/06 15:10:30 [Info] connection established",
    );
    assert_eq!(row.time.as_deref(), Some("2026/06/06 15:10:30"));
    assert_eq!(row.level.as_deref(), Some("Info"));
    assert_eq!(row.component, None);
    assert_eq!(row.message, "connection established");
}

#[test]
fn keeps_unparseable_line_as_raw_message() {
    let row = EngineLogRow::parse("sing-box", ProxyStream::Stderr, "panic: nil map access");
    assert_eq!(row.time, None);
    assert_eq!(row.level, None);
    assert_eq!(row.component, None);
    assert_eq!(row.message, "panic: nil map access");
    assert_eq!(row.stream, ProxyStream::Stderr);
}

#[test]
fn parses_xray_access_log_into_level_and_routing_source() {
    let row = EngineLogRow::parse(
        "xray",
        ProxyStream::Stdout,
        "2026/06/11 03:09:13.163233 from tcp:127.0.0.1:35092 accepted tcp:push.services.mozilla.com:443 [socks-in >> proxy]",
    );
    assert_eq!(row.level.as_deref(), Some("Info"));
    assert_eq!(row.component.as_deref(), Some("socks-in→proxy"));
    assert_eq!(
        row.message,
        "from tcp:127.0.0.1:35092 accepted tcp:push.services.mozilla.com:443"
    );
}

#[test]
fn flags_api_self_traffic_with_arrow_separator() {
    let row = EngineLogRow::parse(
        "xray",
        ProxyStream::Stdout,
        "2026/06/11 15:36:13.086076 from 127.0.0.1:43618 accepted tcp:127.0.0.1:10085 [api -> api]",
    );
    assert_eq!(row.component.as_deref(), Some("api→api"));
    assert!(row.is_self_api_traffic());
}

#[test]
fn does_not_flag_real_traffic_as_self_api() {
    let row = EngineLogRow::parse(
        "xray",
        ProxyStream::Stdout,
        "2026/06/11 03:09:13.163233 from tcp:127.0.0.1:35092 accepted tcp:github.com:443 [socks-in >> proxy]",
    );
    assert!(!row.is_self_api_traffic());
}

#[test]
fn parses_singbox_info_line_with_component() {
    let row = EngineLogRow::parse(
        "sing-box",
        ProxyStream::Stdout,
        "2026-06-06 15:10:29 INFO router: sniffed protocol: tls",
    );
    assert_eq!(row.time.as_deref(), Some("2026-06-06 15:10:29"));
    assert_eq!(row.level.as_deref(), Some("INFO"));
    assert_eq!(row.component.as_deref(), Some("router"));
    assert_eq!(row.message, "sniffed protocol: tls");
}

#[test]
fn parses_singbox_line_with_tz_offset() {
    let row = EngineLogRow::parse(
        "sing-box",
        ProxyStream::Stderr,
        "+0330 2026-06-06 15:10:29 WARN inbound/socks: listen error",
    );
    assert_eq!(row.time.as_deref(), Some("2026-06-06 15:10:29"));
    assert_eq!(row.level.as_deref(), Some("WARN"));
    assert_eq!(row.component.as_deref(), Some("inbound/socks"));
    assert_eq!(row.message, "listen error");
}

#[test]
fn parses_singbox_date_without_time() {
    let row = EngineLogRow::parse(
        "sing-box",
        ProxyStream::Stdout,
        "2026-06-06 INFO router: ready",
    );
    assert_eq!(row.time.as_deref(), Some("2026-06-06"));
    assert_eq!(row.level.as_deref(), Some("INFO"));
    assert_eq!(row.message, "ready");
}

#[test]
fn parses_singbox_level_without_timestamp() {
    let row = EngineLogRow::parse("sing-box", ProxyStream::Stdout, "INFO router: ready");
    assert!(row.time.is_none());
    assert_eq!(row.level.as_deref(), Some("INFO"));
    assert_eq!(row.message, "ready");
}

#[test]
fn keeps_singbox_panic_without_level_as_raw() {
    let row = EngineLogRow::parse("sing-box", ProxyStream::Stderr, "panic: nil map access");
    assert_eq!(row.time, None);
    assert_eq!(row.level, None);
    assert_eq!(row.component, None);
    assert_eq!(row.message, "panic: nil map access");
}

#[test]
fn does_not_treat_multiword_prefix_as_component() {
    let row = EngineLogRow::parse(
        "xray",
        ProxyStream::Stdout,
        "2026/06/06 15:10:31 [Info] rejected: bad request",
    );
    assert_eq!(row.component.as_deref(), Some("rejected"));
    assert_eq!(row.message, "bad request");
}
