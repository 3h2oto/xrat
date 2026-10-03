use super::*;
use xrat_db::NewEvent;

#[test]
fn daemon_unreachable_message_mentions_persistent_install() {
    let message = daemon_unreachable_message(std::path::Path::new("/tmp/xrat.sock"));
    assert!(message.contains("xrat daemon start"));
    assert!(message.contains("xrat daemon install --start"));
    assert!(message.contains("/tmp/xrat.sock"));
}

#[test]
fn no_running_runtime_session_error_is_detected_for_fallback() {
    assert!(is_no_running_runtime_session_error(
        "no running runtime session to replace"
    ));
    assert!(!is_no_running_runtime_session_error(
        "no eligible replacement candidate"
    ));
}

#[test]
fn friendly_result_translates_sentinels() {
    assert_eq!(
        friendly_result("never_triggered"),
        "auto-rotation has not run yet"
    );
    assert_eq!(
        friendly_result("never_selected"),
        "no candidate selected yet"
    );
    assert_eq!(friendly_result("replaced"), "replaced");
}

#[test]
fn parse_rotation_progress_detail_reads_json_fields() {
    assert_eq!(
        parse_rotation_progress_detail(Some(r#"{"done":3,"total":9}"#)),
        Some((3, 9))
    );
}

#[test]
fn parse_rotation_progress_detail_rejects_missing_fields() {
    assert_eq!(parse_rotation_progress_detail(Some(r#"{"done":3}"#)), None);
}

#[test]
fn parse_rotation_progress_detail_rejects_invalid_json() {
    assert_eq!(parse_rotation_progress_detail(Some("not-json")), None);
}

#[test]
fn should_render_rotation_event_filters_expected_sources() {
    let base = NewEvent {
        level: "info".to_string(),
        source: crate::app::events::SOURCE_ROTATION.to_string(),
        kind: "proxy_rotated".to_string(),
        config_id: None,
        session_id: None,
        message: "ok".to_string(),
        detail: None,
    };
    let rotation = EventRecord {
        id: 1,
        level: base.level.clone(),
        source: base.source.clone(),
        kind: base.kind.clone(),
        config_id: base.config_id,
        session_id: base.session_id,
        message: base.message.clone(),
        detail: None,
        created_at: "now".to_string(),
    };
    assert!(should_render_rotation_event(&rotation));

    let subscription = EventRecord {
        source: crate::app::events::SOURCE_SUBSCRIPTION.to_string(),
        kind: "subscription_refresh_result".to_string(),
        ..rotation.clone()
    };
    assert!(should_render_rotation_event(&subscription));

    let test_run = EventRecord {
        source: crate::app::events::SOURCE_TEST.to_string(),
        kind: "test_run".to_string(),
        ..rotation.clone()
    };
    assert!(should_render_rotation_event(&test_run));

    let unrelated = EventRecord {
        source: crate::app::events::SOURCE_DAEMON.to_string(),
        kind: "daemon_started".to_string(),
        ..rotation
    };
    assert!(!should_render_rotation_event(&unrelated));
}

#[test]
fn rotation_config_guidance_lines_include_config_key_and_value() {
    let (config_line, instruction_line) =
        rotation_config_guidance_lines(std::path::Path::new("/tmp/config.toml"), true);
    assert!(config_line.contains("Config file: "));
    assert!(config_line.contains("/tmp/config.toml"));
    assert!(instruction_line.contains("[runtime.rotation].enabled = true"));
}
