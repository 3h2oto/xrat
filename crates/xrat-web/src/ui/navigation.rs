use dioxus::prelude::*;
use icons::{
    FileCode, Globe, Layers, LayoutDashboard, ListChecks, RefreshCw, Rss, ScrollText, Search,
    Settings2, Shield, Terminal, UserRound, Users, Wrench,
};

#[component]
pub fn PageIcon(title: String) -> Element {
    match title.as_str() {
        "Overview" => rsx! {
            LayoutDashboard {}
        },
        "Configs" => rsx! {
            Layers {}
        },
        "Subscriptions" => rsx! {
            Rss {}
        },
        "Jobs" => rsx! {
            ListChecks {}
        },
        "Logs" | "XRAT logs" => rsx! {
            ScrollText {}
        },
        "Engine logs" => rsx! {
            Terminal {}
        },
        "TCP scanner" => rsx! {
            Search {}
        },
        "Parse configs" => rsx! {
            FileCode {}
        },
        "GeoIP lookup" => rsx! {
            Globe {}
        },
        "Rotation" => rsx! {
            RefreshCw {}
        },
        "Host administration" => rsx! {
            Shield {}
        },
        "Tools" => rsx! {
            Wrench {}
        },
        "Settings" => rsx! {
            Settings2 {}
        },
        "Users" => rsx! {
            Users {}
        },
        _ => rsx! {
            UserRound {}
        },
    }
}

pub fn page_caption(title: &str) -> &'static str {
    match title {
        "Overview" => "Your connections and runtime at a glance.",
        "Subscriptions" => "Manage subscription sources and refresh their configurations.",
        "Configs" => "Browse, test, and connect your configurations.",
        "Jobs" => "Follow background tasks and inspect their results.",
        "Logs" | "XRAT logs" => "Inspect XRAT activity and operational events.",
        "Engine logs" => "Inspect output from your latest engine session.",
        "TCP scanner" => "Find reachable addresses from the XRAT host.",
        "Parse configs" => "Check configuration links without saving them.",
        "GeoIP lookup" => "Look up geographic information for an address or hostname.",
        "Host administration" => {
            "Manage the machine running XRAT. Changes also apply when connected remotely."
        }
        "Tools" => "Utilities for testing addresses and inspecting configurations.",
        "Settings" => "Configure how XRAT runs on this host.",
        "Users" => "Manage accounts, roles, and access to this workspace.",
        "Account" => "Manage your password and API tokens.",
        _ => "",
    }
}
