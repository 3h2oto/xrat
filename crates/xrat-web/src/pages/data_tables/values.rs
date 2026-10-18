use super::state::TableState;
use std::cmp::Ordering;
use xrat_web::types::{ConfigRow, SubscriptionRow};

#[derive(Clone, PartialEq)]
pub enum CellValue {
    Text(String),
    Number(f64, &'static str),
    Probe {
        ok: Option<bool>,
        value: Option<f64>,
        detail: Option<String>,
    },
    Outcome {
        state: &'static str,
        detail: Option<String>,
    },
    Empty,
}
impl CellValue {
    fn missing(&self) -> bool {
        matches!(
            self,
            Self::Empty
                | Self::Probe {
                    ok: Some(false),
                    ..
                }
                | Self::Probe {
                    ok: None,
                    value: None,
                    ..
                }
        )
    }
    pub fn compare(&self, other: &Self, descending: bool) -> Ordering {
        match (self.missing(), other.missing()) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            _ => {}
        }
        let order = match (self, other) {
            (Self::Number(a, _), Self::Number(b, _)) => a.total_cmp(b),
            (Self::Probe { value: a, .. }, Self::Probe { value: b, .. }) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            _ => self.text().to_lowercase().cmp(&other.text().to_lowercase()),
        };
        if descending { order.reverse() } else { order }
    }
    pub fn text(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Number(value, unit) => {
                if *unit == "Mbps" {
                    format!("{value:.2} {unit}")
                } else if unit.is_empty() {
                    format!("{value:.0}")
                } else {
                    format!("{value:.0} {unit}")
                }
            }
            Self::Probe {
                ok: Some(false), ..
            } => "Failed".into(),
            Self::Probe {
                value: Some(value), ..
            } => format!("{value:.0} ms"),
            Self::Probe { ok: Some(true), .. } => "Passed".into(),
            Self::Outcome { state, .. } => match *state {
                "passed" => "Passed",
                "failed" => "Failed",
                _ => "Not tested",
            }
            .into(),
            _ => String::new(),
        }
    }
}
fn text(value: Option<&str>) -> CellValue {
    value
        .filter(|s| !s.is_empty())
        .map(|s| CellValue::Text(s.into()))
        .unwrap_or(CellValue::Empty)
}
fn number(value: Option<f64>, unit: &'static str) -> CellValue {
    value
        .filter(|value| value.is_finite())
        .map(|value| CellValue::Number(value, unit))
        .unwrap_or(CellValue::Empty)
}
pub fn config_name(row: &ConfigRow) -> String {
    if row.name.is_empty() {
        format!("Config #{}", row.id)
    } else {
        row.name.clone()
    }
}
pub fn config_state(row: &ConfigRow) -> &'static str {
    if row.deleted {
        "Deleted"
    } else if row.enabled {
        "Enabled"
    } else {
        "Disabled"
    }
}
pub fn test_state(row: &ConfigRow) -> &'static str {
    let Some(test) = &row.test else {
        return "untested";
    };
    let ok = test.real_delay_ok.or(test.tcp_ok).or(test.icmp_ok);
    if ok == Some(true) {
        "passed"
    } else if ok == Some(false) || test.failure_reason.is_some() {
        "failed"
    } else if test.download_mbps.is_some() || test.upload_mbps.is_some() {
        "passed"
    } else {
        "untested"
    }
}
pub fn config_value(row: &ConfigRow, key: &str, subs: &[SubscriptionRow]) -> CellValue {
    let test = row.test.as_ref();
    match key {
        "name" => CellValue::Text(config_name(row)),
        "state" => CellValue::Text(config_state(row).into()),
        "protocol" => text(Some(&row.protocol)),
        "address" => text(Some(&row.address)),
        "port" => number(Some(row.port as f64), ""),
        "id" => number(Some(row.id as f64), ""),
        "subscription" => text(
            row.subscription_id
                .and_then(|id| subs.iter().find(|sub| sub.id == id))
                .map(|sub| sub.name.as_str()),
        ),
        "network" => text(Some(&row.network)),
        "tls" => text(row.tls.as_deref()),
        "sni" => text(row.sni.as_deref()),
        "country" => text(row.country.as_deref()),
        "asn" => text(row.asn.as_deref()),
        "updated" => text(Some(&row.updated_at)),
        "tested" => text(test.and_then(|test| test.tested_at.as_deref())),
        "test" => CellValue::Outcome {
            state: test_state(row),
            detail: test.and_then(|test| test.failure_reason.clone()),
        },
        "delay" | "tcp" | "icmp" => {
            let (ok, value) = test
                .map(|test| match key {
                    "tcp" => (test.tcp_ok, test.tcp_ms),
                    "icmp" => (test.icmp_ok, test.icmp_ms),
                    _ => (test.real_delay_ok, test.real_delay_ms),
                })
                .unwrap_or((None, None));
            CellValue::Probe {
                ok,
                value: value.map(|value| value as f64),
                detail: test.and_then(|test| test.failure_reason.clone()),
            }
        }
        "download" => number(test.and_then(|test| test.download_mbps), "Mbps"),
        "upload" => number(test.and_then(|test| test.upload_mbps), "Mbps"),
        "connect" => number(
            test.and_then(|test| test.connect_ms)
                .map(|value| value as f64),
            "ms",
        ),
        "ttfb" => number(
            test.and_then(|test| test.ttfb_ms).map(|value| value as f64),
            "ms",
        ),
        "http" => number(
            test.and_then(|test| test.http_status)
                .map(|value| value as f64),
            "",
        ),
        _ => CellValue::Empty,
    }
}
pub fn source_label(source: &str) -> String {
    match source {
        "url" => "URL".into(),
        "file" => "File".into(),
        "raw_text" => "Imported text".into(),
        other => other.replace('_', " "),
    }
}
pub fn subscription_value(row: &SubscriptionRow, key: &str) -> CellValue {
    match key {
        "name" => text(Some(&row.name)),
        "source" => CellValue::Text(source_label(&row.source)),
        "count" => number(Some(row.config_count as f64), ""),
        "created" => text(Some(&row.created_at)),
        "updated" => text(Some(&row.updated_at)),
        "id" => number(Some(row.id as f64), ""),
        _ => CellValue::Empty,
    }
}
pub fn filtered_configs<'a>(
    rows: &'a [ConfigRow],
    subs: &[SubscriptionRow],
    state: &TableState,
) -> Vec<&'a ConfigRow> {
    let needle = state.query.trim().to_lowercase();
    let mut filtered = rows
        .iter()
        .filter(|row| {
            format!(
                "{} {} {} {}",
                config_name(row),
                row.protocol,
                row.address,
                row.id
            )
            .to_lowercase()
            .contains(&needle)
                && (state.show_deleted || !row.deleted)
                && (state.protocol.is_empty() || row.protocol == state.protocol)
                && (state.state.is_empty() || config_state(row).eq_ignore_ascii_case(&state.state))
                && (state.subscription.is_empty()
                    || if state.subscription == "none" {
                        row.subscription_id.is_none()
                    } else {
                        row.subscription_id.map(|id| id.to_string()).as_deref()
                            == Some(&state.subscription)
                    })
                && (state.test.is_empty() || test_state(row) == state.test)
                && (state.country.is_empty() || row.country.as_deref() == Some(&state.country))
                && state.max_delay.parse::<u64>().ok().is_none_or(|max| {
                    row.test.as_ref().is_some_and(|test| {
                        test.real_delay_ok != Some(false)
                            && test
                                .real_delay_ms
                                .is_some_and(|delay| delay >= 0 && delay as u64 <= max)
                    })
                })
        })
        .collect::<Vec<_>>();
    filtered.sort_by(|a, b| {
        config_value(a, &state.sort, subs)
            .compare(&config_value(b, &state.sort, subs), state.descending)
            .then_with(|| a.id.cmp(&b.id))
    });
    filtered
}
pub fn filtered_subscriptions<'a>(
    rows: &'a [SubscriptionRow],
    state: &TableState,
) -> Vec<&'a SubscriptionRow> {
    let needle = state.query.trim().to_lowercase();
    let mut filtered = rows
        .iter()
        .filter(|row| {
            format!("{} {} {}", row.name, row.source, row.id)
                .to_lowercase()
                .contains(&needle)
                && (state.source.is_empty() || row.source == state.source)
        })
        .collect::<Vec<_>>();
    filtered.sort_by(|a, b| {
        subscription_value(a, &state.sort)
            .compare(&subscription_value(b, &state.sort), state.descending)
            .then_with(|| a.id.cmp(&b.id))
    });
    filtered
}
pub fn csv(rows: &[Vec<String>]) -> String {
    rows.iter()
        .map(|row| {
            row.iter()
                .map(|value| {
                    let safe = if value.trim_start().starts_with(['=', '+', '-', '@'])
                        || value.starts_with(['\t', '\r', '\n'])
                    {
                        format!("'{value}")
                    } else {
                        value.clone()
                    };
                    format!("\"{}\"", safe.replace('"', "\"\""))
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\r\n")
}
