use super::state::TableState;
use xrat_web::types::{ConfigRow, SubscriptionRow};

pub use xrat_web::configs::values::*;

pub fn filtered_configs<'a>(
    rows: &'a [ConfigRow],
    subs: &[SubscriptionRow],
    state: &TableState,
    active_config: Option<i64>,
) -> Vec<&'a ConfigRow> {
    xrat_web::configs::sharing::filter_configs(rows, subs, &state.share_filter(), active_config)
}

pub fn filtered_subscriptions<'a>(
    rows: &'a [SubscriptionRow],
    state: &TableState,
) -> Vec<&'a SubscriptionRow> {
    let needle = state.query.trim().to_lowercase();
    let mut filtered = rows
        .iter()
        .filter(|row| {
            format!("{} {} {}", row.name, row.source, row.reference)
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
