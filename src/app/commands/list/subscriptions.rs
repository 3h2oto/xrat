use super::configs::*;
use super::prelude::*;
use crate::app::commands::output;

pub(crate) fn format_subscriptions(
    subscriptions: &[SubscriptionRecord],
    format: ListFormat,
) -> crate::app::Result<String> {
    match format {
        ListFormat::Table => Ok(format_subscription_table(subscriptions)),
        ListFormat::Tsv => Ok(format_subscription_tsv(subscriptions)),
        ListFormat::Json => Ok(serde_json::to_string_pretty(
            &subscriptions
                .iter()
                .map(subscription_json)
                .collect::<Vec<_>>(),
        )?),
    }
}

pub(crate) fn format_subscription_table(subscriptions: &[SubscriptionRecord]) -> String {
    let columns = [
        Column {
            header: "REF",
            align: Align::Left,
        },
        Column {
            header: "KIND",
            align: Align::Left,
        },
        Column {
            header: "CONFIGS",
            align: Align::Right,
        },
        Column {
            header: "NAME",
            align: Align::Left,
        },
        Column {
            header: "SOURCE",
            align: Align::Left,
        },
        Column {
            header: "UPDATED AT",
            align: Align::Left,
        },
    ];
    let rows = subscriptions
        .iter()
        .map(|subscription| {
            vec![
                Cell::plain(short_ref(&subscription.r#ref).to_string()),
                Cell::plain(subscription.source_kind.clone()),
                Cell::plain(subscription.config_count.to_string()),
                Cell::plain(output::truncate(
                    subscription.name.as_deref().unwrap_or("-"),
                    24,
                )),
                Cell::plain(output::truncate(
                    subscription.source_url.as_deref().unwrap_or("-"),
                    56,
                )),
                Cell::plain(subscription.updated_at.clone()),
            ]
        })
        .collect::<Vec<_>>();

    output::format_table(&columns, &rows, output::color_enabled())
}

pub(crate) fn format_subscription_tsv(subscriptions: &[SubscriptionRecord]) -> String {
    let mut lines = Vec::with_capacity(subscriptions.len() + 1);
    lines.push("ref\tkind\tconfig_count\tname\tsource\tupdated_at".to_string());
    for subscription in subscriptions {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            subscription.r#ref,
            subscription.source_kind,
            subscription.config_count,
            tsv_cell(subscription.name.as_deref()),
            tsv_cell(subscription.source_url.as_deref()),
            subscription.updated_at,
        ));
    }
    lines.join("\n")
}
