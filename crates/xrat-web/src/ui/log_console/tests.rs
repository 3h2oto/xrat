use super::*;
use parser::{parse_engine_line, severity, strip_ansi};

#[test]
fn recognizes_xray_and_singbox_prefixes_without_message_based_false_positives() {
    let xray = parse_engine_line("2026/10/18 12:34:56.123456 [Warning] proxy: connection closed");
    assert_eq!(xray.level, "warn");
    assert_eq!(
        xray.normalized_time.as_deref(),
        Some("2026-10-18T12:34:56.123456")
    );
    assert_eq!(xray.message, "proxy: connection closed");
    let sing = parse_engine_line("+0330 2026-10-18 12:34:56 ERROR [42 3ms] outbound: failed");
    assert_eq!(sing.level, "error");
    assert_eq!(
        sing.normalized_time.as_deref(),
        Some("2026-10-18T12:34:56+03:30")
    );
    assert_eq!(sing.message, "[42 3ms] outbound: failed");
    let info = parse_engine_line("2026-10-18T09:04:56Z INFO no error reported");
    assert_eq!(info.level, "info");
    assert_eq!(
        parse_engine_line("ordinary message about an error").level,
        "unknown"
    );
    assert_eq!(
        parse_engine_line("INFO[0042] router: started").level,
        "info"
    );
}
#[test]
fn removes_ansi_colour_and_terminal_link_sequences() {
    assert_eq!(
        strip_ansi("\u{1b}[31mERROR\u{1b}[0m failure\r"),
        "ERROR failure"
    );
    assert_eq!(
        strip_ansi("\u{1b}]8;;https://example.test\u{1b}\\label\u{1b}]8;;\u{1b}\\"),
        "label"
    );
    assert_eq!(severity("[Fatal]"), Some("error"));
    assert_eq!(severity("[DEBUG]"), Some("debug"));
}
#[test]
fn groups_untimed_stack_continuations_and_preserves_raw_lines() {
    let lines = engine_lines(
        "ERROR failed\n  at worker.rs:42\nCaused by: timeout\nINFO recovered",
        "Session 2 · err.log",
    );
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].message, "recovered");
    assert_eq!(lines[1].source, "stderr");
    assert_eq!(
        lines[1].message,
        "failed\n  at worker.rs:42\nCaused by: timeout"
    );
    assert!(lines[1].text.contains("ERROR failed\n  at worker.rs:42"));
}
#[test]
fn orders_merged_streams_newest_first_with_unknown_times_last() {
    let mut lines = vec![
        LogLine {
            time: Some(10.0),
            text: "old".into(),
            ..Default::default()
        },
        LogLine {
            text: "untimed".into(),
            ..Default::default()
        },
        LogLine {
            time: Some(30.0),
            text: "new".into(),
            ..Default::default()
        },
        LogLine {
            time: Some(20.0),
            text: "middle".into(),
            ..Default::default()
        },
    ];
    newest_first(&mut lines);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>(),
        ["new", "middle", "old", "untimed"]
    );
}
#[test]
fn filters_combine_source_severity_and_case_insensitive_message_search() {
    let line = LogLine {
        source: "stderr".into(),
        level: "warn".into(),
        message: "Connection closed".into(),
        timestamp: "12:34:56".into(),
        ..Default::default()
    };
    assert!(matches_filters(&line, " CONNECTION ", "warn", "stderr"));
    assert!(!matches_filters(&line, "connection", "error", "stderr"));
    assert!(!matches_filters(&line, "connection", "all", "stdout"));
    assert!(matches_filters(&line, "12:34", "all", "all"));
}
#[test]
fn stable_keys_survive_new_entries_and_distinguish_duplicate_lines() {
    let old = LogLine {
        text: "old".into(),
        ..Default::default()
    };
    let new = LogLine {
        text: "new".into(),
        ..Default::default()
    };
    let previous = keyed_entries(&[&old, &old]);
    let updated = keyed_entries(&[&new, &old, &old]);
    assert_ne!(previous[0].0, previous[1].0);
    assert_eq!(previous[0].0, updated[1].0);
    assert_eq!(previous[1].0, updated[2].0);
}
#[test]
fn explicit_clock_keeps_a_paused_relative_window_from_advancing() {
    fn check() -> Element {
        let window = use_time_window("0.25");
        assert!(window.contains_at(9_500.0, 10_000.0));
        assert!(!window.contains_at(9_500.0, 11_000.0));
        assert!(!window.contains_at(10_001.0, 10_000.0));
        rsx! {
            div {}
        }
    }
    let mut dom = VirtualDom::new(check);
    dom.rebuild_to_vec();
}
