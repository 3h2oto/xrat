use super::*;
use crate::types::{ConfigTest, SubscriptionRow};

fn rows() -> Vec<ConfigRow> {
    vec![
        ConfigRow {
            id: 1,
            name: "Alpha".into(),
            reference: "aaa".into(),
            protocol: "vless".into(),
            address: "one.example".into(),
            enabled: true,
            country: Some("DE".into()),
            subscription_id: Some(7),
            test: Some(ConfigTest {
                real_delay_ok: Some(true),
                real_delay_ms: Some(30),
                ..Default::default()
            }),
            ..Default::default()
        },
        ConfigRow {
            id: 2,
            name: "Beta".into(),
            reference: "bbb".into(),
            protocol: "ss".into(),
            enabled: false,
            country: Some("FR".into()),
            test: Some(ConfigTest {
                real_delay_ok: Some(false),
                failure_reason: Some("timeout".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
        ConfigRow {
            id: 3,
            name: "Gamma".into(),
            protocol: "vless".into(),
            enabled: true,
            deleted: true,
            ..Default::default()
        },
    ]
}
fn ids(filter: ConfigFilter) -> Vec<i64> {
    filter_configs(
        &rows(),
        &[SubscriptionRow {
            id: 7,
            reference: "sub-seven".into(),
            ..Default::default()
        }],
        &filter,
        None,
    )
    .iter()
    .map(|row| row.id)
    .collect()
}
#[test]
fn filters_cover_every_config_control_and_deleted_state() {
    assert_eq!(ids(ConfigFilter::default()), vec![1, 2]);
    assert_eq!(
        ids(ConfigFilter {
            q: "ONE.EXAMPLE".into(),
            ..Default::default()
        }),
        vec![1]
    );
    assert_eq!(
        ids(ConfigFilter {
            protocol: "ss".into(),
            ..Default::default()
        }),
        vec![2]
    );
    assert_eq!(
        ids(ConfigFilter {
            state: "disabled".into(),
            ..Default::default()
        }),
        vec![2]
    );
    assert_eq!(
        ids(ConfigFilter {
            subscription: "sub-seven".into(),
            ..Default::default()
        }),
        vec![1]
    );
    assert_eq!(
        ids(ConfigFilter {
            subscription: "none".into(),
            ..Default::default()
        }),
        vec![2]
    );
    assert_eq!(
        ids(ConfigFilter {
            test: "failed".into(),
            ..Default::default()
        }),
        vec![2]
    );
    assert_eq!(
        ids(ConfigFilter {
            test: "untested".into(),
            show_deleted: true,
            ..Default::default()
        }),
        vec![3]
    );
    assert_eq!(
        ids(ConfigFilter {
            country: "DE".into(),
            max_delay: Some(30),
            test: "passed".into(),
            ..Default::default()
        }),
        vec![1]
    );
    assert!(
        ids(ConfigFilter {
            max_delay: Some(29),
            ..Default::default()
        })
        .is_empty()
    );
    assert_eq!(
        ids(ConfigFilter {
            show_deleted: true,
            ..Default::default()
        }),
        vec![1, 2, 3]
    );
    assert_eq!(
        ids(ConfigFilter {
            state: "deleted".into(),
            show_deleted: true,
            ..Default::default()
        }),
        vec![3]
    );
    assert!(
        ids(ConfigFilter {
            state: "deleted".into(),
            ..Default::default()
        })
        .is_empty()
    );
}
#[test]
fn sorting_and_invalid_filters_are_explicit() {
    assert_eq!(
        ids(ConfigFilter {
            order: "desc".into(),
            sort: "name".into(),
            ..Default::default()
        }),
        vec![2, 1]
    );
    for filter in [
        ConfigFilter {
            state: "typo".into(),
            ..Default::default()
        },
        ConfigFilter {
            test: "typo".into(),
            ..Default::default()
        },
        ConfigFilter {
            sort: "typo".into(),
            ..Default::default()
        },
        ConfigFilter {
            order: "typo".into(),
            ..Default::default()
        },
    ] {
        assert!(filter.validate().is_err());
    }
}

#[test]
fn multiple_values_use_or_within_filters_and_and_across_filters() {
    let filter = ConfigFilter {
        protocol: "ss,vless".into(),
        state: "disabled,enabled".into(),
        country: "DE,FR".into(),
        test: "failed,passed".into(),
        subscription: "none,sub-seven".into(),
        ..Default::default()
    };
    assert!(filter.validate().is_ok());
    assert_eq!(ids(filter.clone()), vec![1, 2]);
    assert_eq!(
        ids(ConfigFilter {
            country: "DE".into(),
            ..filter.clone()
        }),
        vec![1]
    );
    assert_eq!(
        ids(ConfigFilter {
            protocol: "trojan,ss".into(),
            ..filter.clone()
        }),
        vec![2]
    );
    assert!(
        ConfigFilter {
            state: "enabled,unknown".into(),
            ..filter.clone()
        }
        .validate()
        .is_err()
    );
    assert!(
        ConfigFilter {
            test: "passed,unknown".into(),
            ..filter
        }
        .validate()
        .is_err()
    );
}
