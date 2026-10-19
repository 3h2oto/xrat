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
        if !["", "all", "enabled", "disabled", "deleted"].contains(&self.state.as_str()) {
            return Err("Invalid config state");
        }
        if !["", "passed", "failed", "untested"].contains(&self.test.as_str()) {
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
                && (filter.protocol.is_empty() || row.protocol == filter.protocol)
                && (filter.state.is_empty()
                    || filter.state == "all"
                    || config_state(row).eq_ignore_ascii_case(&filter.state))
                && (filter.subscription.is_empty()
                    || if filter.subscription == "none" {
                        row.subscription_id.is_none()
                    } else {
                        row.subscription_id
                            .and_then(|id| subs.iter().find(|sub| sub.id == id))
                            .is_some_and(|sub| sub.reference == filter.subscription)
                    })
                && (filter.test.is_empty() || test_state(row) == filter.test)
                && (filter.country.is_empty() || row.country.as_deref() == Some(&filter.country))
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
