use crate::{
    configs::values::*,
    types::{ConfigRow, SubscriptionRow},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigFilter {
    pub q: String,
    pub protocol: String,
    pub state: String,
    pub subscription: String,
    pub test: String,
    pub country: String,
    pub max_delay: Option<u64>,
    pub show_deleted: bool,
    pub sort: String,
    pub order: String,
}

impl ConfigFilter {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !filter_values(&self.state)
            .all(|value| ["all", "enabled", "disabled", "deleted"].contains(&value))
        {
            return Err("Invalid config state");
        }
        if !filter_values(&self.test).all(|value| ["passed", "failed", "untested"].contains(&value))
        {
            return Err("Invalid test result");
        }
        if !["", "asc", "desc"].contains(&self.order.as_str()) {
            return Err("Invalid sort order");
        }
        if ![
            "",
            "name",
            "state",
            "protocol",
            "address",
            "port",
            "ref",
            "subscription",
            "network",
            "tls",
            "sni",
            "country",
            "asn",
            "updated",
            "tested",
            "test",
            "delay",
            "tcp",
            "icmp",
            "download",
            "upload",
            "connect",
            "ttfb",
            "http",
        ]
        .contains(&self.sort.as_str())
        {
            return Err("Invalid sort column");
        }
        Ok(())
    }
}

pub fn filter_values(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').filter(|value| !value.is_empty())
}

pub fn filter_matches(selection: &str, value: &str) -> bool {
    selection.is_empty() || filter_values(selection).any(|selected| selected == value)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum ConfigShareRequest {
    Selected { ids: Vec<i64> },
    Filtered { filter: ConfigFilter },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigShareResponse {
    pub raw: String,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSharePreview {
    pub count: usize,
    pub qr_supported: bool,
}

pub fn filter_configs<'a>(
    rows: &'a [ConfigRow],
    subs: &[SubscriptionRow],
    filter: &ConfigFilter,
    active: Option<i64>,
) -> Vec<&'a ConfigRow> {
    let needle = filter.q.trim().to_lowercase();
    let mut matching = rows
        .iter()
        .filter(|row| {
            format!(
                "{} {} {} {}",
                config_name(row),
                row.protocol,
                row.address,
                row.reference
            )
            .to_lowercase()
            .contains(&needle)
                && (filter.show_deleted || !row.deleted)
                && filter_matches(&filter.protocol, &row.protocol)
                && (filter_values(&filter.state).any(|value| value == "all")
                    || filter_matches(&filter.state, &config_state(row).to_lowercase()))
                && filter_matches(
                    &filter.subscription,
                    row.subscription_id
                        .and_then(|id| subs.iter().find(|sub| sub.id == id))
                        .map(|sub| sub.reference.as_str())
                        .unwrap_or("none"),
                )
                && filter_matches(&filter.test, test_state(row))
                && filter_matches(&filter.country, row.country.as_deref().unwrap_or_default())
                && filter.max_delay.is_none_or(|max| {
                    row.test.as_ref().is_some_and(|test| {
                        test.real_delay_ok != Some(false)
                            && test
                                .real_delay_ms
                                .is_some_and(|delay| delay >= 0 && delay as u64 <= max)
                    })
                })
        })
        .collect::<Vec<_>>();
    matching.sort_by(|a, b| {
        (Some(b.id) == active)
            .cmp(&(Some(a.id) == active))
            .then_with(|| {
                config_value(
                    a,
                    if filter.sort.is_empty() {
                        "name"
                    } else {
                        &filter.sort
                    },
                    subs,
                )
                .compare(
                    &config_value(
                        b,
                        if filter.sort.is_empty() {
                            "name"
                        } else {
                            &filter.sort
                        },
                        subs,
                    ),
                    filter.order == "desc",
                )
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    matching
}

#[cfg(test)]
mod tests;
