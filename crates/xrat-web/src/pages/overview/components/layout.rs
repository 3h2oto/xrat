use crate::ui::card::Card;
use dioxus::prelude::*;

const SUMMARY_CLASSES: &str = concat!(
    "dashboard-summary-card flex flex-col shadow-none rounded-[10px] ",
    "border-[color:color-mix(in_srgb,_var(--border)_40%,_transparent)] py-3.5 px-4 gap-2.5 ",
    "[&_h3]:flex [&_h3]:items-center [&_h3]:text-[13px] [&_h3]:font-semibold [&_h3]:tracking-[0] ",
    "[&_h3]:m-0 [&_h3]:gap-[7px] [&_h3_svg]:w-[15px] [&_h3_svg]:h-[15px] ",
    "[&_h3_svg]:text-muted-foreground",
);
const CHART_CLASSES: &str = concat!(
    "dashboard-chart-card [&_h3]:flex [&_h3]:items-center [&_h3]:text-[13px] [&_h3]:font-semibold ",
    "[&_h3]:tracking-[0] [&_h3]:m-0 [&_h3]:gap-[7px] [&_h3_svg]:size-[15px] ",
    "[&_h3_svg]:text-muted-foreground shadow-none rounded-[10px] ",
    "border-[color:color-mix(in_srgb,_var(--border)_40%,_transparent)] py-3.5 px-4 gap-2 ",
    "min-[1001px]:h700:flex-col ",
    "min-[1001px]:h700:min-h-0 [@media(width<=600px)]:pr-3.5 ",
    "[@media(width<=600px)]:pl-3.5 min-[1001px]:h700:flex ",
    "min-[1001px]:h700:[&_>_:not(.dashboard-plot)]:shrink-[0]",
);

#[component]
pub fn SummaryCard(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    rsx! {
        Card { class: tw_merge::tw_merge!(SUMMARY_CLASSES, class.as_deref().unwrap_or("")), {children} }
    }
}

#[component]
pub fn ChartCard(children: Element) -> Element {
    rsx! { Card { class: CHART_CLASSES, {children} } }
}
pub const CHART_TONES: [&str; 3] = [
    "[--series-color:var(--chart-0)]",
    "[--series-color:var(--chart-1)]",
    "[--series-color:var(--chart-2)]",
];

pub const HEALTH_COLORS: [&str; 5] = [
    "text-[color:var(--chart-0)]",
    "text-[color:var(--chart-1)]",
    "text-[color:var(--chart-2)]",
    "text-destructive",
    "text-[color:color-mix(in_srgb,var(--muted)_30%,var(--panel))]",
];
