use super::{bulk::toggle_page, state::*, values::*};
use std::collections::BTreeSet;
use xrat_web::types::{ConfigRow, ConfigTest, SubscriptionRow};

#[test]
fn view_query_round_trips_filters_sort_columns_and_pagination() {
    let query = "q=hello+world%26x&protocol=vless&state=enabled&subscription=12&test=failed&max_delay=120&country=DE&sort=tcp&order=desc&columns=tcp,download,id&page=3&per_page=50";
    let state = TableState::parse(query, false);
    assert_eq!(state.query, "hello world&x");
    assert_eq!(state, TableState::parse(&state.query_string(false), false));
    let state = TableState::parse(
        "q=two&source=url&columns=updated&sort=count&order=desc&page=2&per_page=100",
        true,
    );
    assert_eq!(state, TableState::parse(&state.query_string(true), true));
    assert!(TableState::new(false).query_string(false).is_empty());
    assert!(TableState::parse("columns=", false).columns.is_empty());
}
#[test]
fn invalid_query_values_are_bounded_and_unknown_columns_cannot_be_injected() {
    let state = TableState::parse(
        "sort=password&columns=password,tcp&state=other&test=whatever&page=0&per_page=900&max_delay=-1",
        false,
    );
    assert_eq!(state.sort, "name");
    assert_eq!(state.columns, BTreeSet::from(["tcp".into()]));
    assert_eq!(state.page, 1);
    assert_eq!(state.per_page, 25);
    assert!(state.max_delay.is_empty());
    assert!(state.state.is_empty());
    assert!(state.test.is_empty());
}
fn config(id: i64, delay: Option<i64>, ok: Option<bool>) -> ConfigRow {
    ConfigRow {
        id,
        name: format!("Node {id}"),
        protocol: "vless".into(),
        address: "example.test".into(),
        enabled: true,
        test: delay.or(ok.map(|_| 0)).map(|_| ConfigTest {
            real_delay_ok: ok,
            real_delay_ms: delay,
            ..Default::default()
        }),
        ..Default::default()
    }
}
#[test]
fn numeric_sort_keeps_failed_and_missing_tests_last_in_both_directions() {
    let rows = vec![
        config(1, Some(120), Some(true)),
        config(2, Some(9), Some(true)),
        config(3, None, None),
        config(4, Some(0), Some(false)),
    ];
    let mut state = TableState::new(false);
    state.sort = "delay".into();
    assert_eq!(
        filtered_configs(&rows, &[], &state)
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        [2, 1, 3, 4]
    );
    state.descending = true;
    assert_eq!(
        filtered_configs(&rows, &[], &state)
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>()[..2],
        [1, 2]
    );
}
#[test]
fn combined_config_filters_exclude_failed_latency_and_deleted_rows() {
    let mut rows = vec![
        config(1, Some(90), Some(true)),
        config(2, Some(20), Some(false)),
        config(3, Some(90), Some(true)),
    ];
    for row in &mut rows {
        row.subscription_id = Some(8);
        row.country = Some("DE".into());
    }
    rows[2].deleted = true;
    let state = TableState::parse(
        "q=node&protocol=vless&subscription=8&country=DE&state=enabled&test=passed&max_delay=100",
        false,
    );
    assert_eq!(
        filtered_configs(&rows, &[], &state)
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(test_state(&rows[1]), "failed");
}
#[test]
fn subscription_count_sort_is_numeric_and_source_filter_applies() {
    let rows = vec![
        SubscriptionRow {
            id: 1,
            source: "url".into(),
            config_count: 100,
            ..Default::default()
        },
        SubscriptionRow {
            id: 2,
            source: "url".into(),
            config_count: 9,
            ..Default::default()
        },
        SubscriptionRow {
            id: 3,
            source: "raw_text".into(),
            ..Default::default()
        },
    ];
    let state = TableState::parse("source=url&sort=count", true);
    assert_eq!(
        filtered_subscriptions(&rows, &state)
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        [2, 1]
    );
}
#[test]
fn page_selection_retains_other_pages_and_toggles_only_current_page() {
    let mut selected = BTreeSet::from([99]);
    toggle_page(&mut selected, &[1, 2]);
    assert_eq!(selected, BTreeSet::from([1, 2, 99]));
    toggle_page(&mut selected, &[1, 2]);
    assert_eq!(selected, BTreeSet::from([99]));
}
#[test]
fn csv_quotes_fields_and_neutralizes_spreadsheet_formulas() {
    let output = csv(&[vec!["=SUM(A1:A2)".into(), "hello,\"world\"\nline".into()]]);
    assert_eq!(output, "\"'=SUM(A1:A2)\",\"hello,\"\"world\"\"\nline\"");
}

#[test]
fn csv_neutralizes_formulas_after_leading_whitespace() {
    let output = csv(&[vec!["  =SUM(A1:A2)".into(), "\n@formula".into()]]);
    assert_eq!(output, "\"'  =SUM(A1:A2)\",\"'\n@formula\"");
}

#[test]
fn deleted_visibility_defaults_off_and_round_trips_in_query() {
    let mut deleted = config(2, None, None);
    deleted.deleted = true;
    let rows = vec![config(1, None, None), deleted];
    let state = TableState::new(false);
    assert_eq!(filtered_configs(&rows, &[], &state).len(), 1);
    let state = TableState::parse("show_deleted=true", false);
    assert_eq!(filtered_configs(&rows, &[], &state).len(), 2);
    assert_eq!(state, TableState::parse(&state.query_string(false), false));
    assert_ne!(state.filter_key(), TableState::new(false).filter_key());
    let state = TableState::parse("state=deleted", false);
    assert!(state.show_deleted);
    assert_eq!(filtered_configs(&rows, &[], &state)[0].id, 2);
    assert!(!TableState::parse("show_deleted=invalid", false).show_deleted);
}
