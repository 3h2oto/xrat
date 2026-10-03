use super::*;

#[test]
fn normalizes_and_formats_tui_test_stages() {
    let stages = normalize_test_stage_names(&[
        "icmp".to_string(),
        "real-delay".to_string(),
        "icmp".to_string(),
    ]);

    assert_eq!(stages, vec!["icmp", "real_delay"]);
    assert_eq!(format_test_stage_label(&stages), "icmp + real-delay");
}
