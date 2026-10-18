use crate::ui::{
    button::{Button, ButtonVariant},
    card::Card,
    form_select::{FormSelect, FormSelectOption},
    input::{Input, InputType},
    label::Label,
    spinner::Spinner,
    states::{EmptyState, ErrorState, LoadingState},
    switch::Switch,
    textarea::Textarea,
    tooltip::{Tooltip, TooltipContent, TooltipPosition},
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{
    Check, Clock, Cpu, FileCode, FlaskConical, Globe, Lock, RefreshCw, RotateCcw, Route, Rss, Save,
    Search, Server, Terminal, X,
};
use std::collections::BTreeMap;
use xrat_web::types::{Setting, SettingChange, Settings, SettingsBatchUpdate};

#[component]
pub fn SettingsPage() -> Element {
    let mut settings = use_signal(|| None::<Settings>);
    let mut loading = use_signal(|| true);
    let mut load_error = use_signal(String::new);
    let mut category = use_signal(|| "runtime".to_string());
    let mut query = use_signal(String::new);
    let mut drafts = use_signal(BTreeMap::<String, String>::new);
    let mut saving = use_signal(|| None::<String>);
    let mut reset = use_signal(|| 0_u64);
    let Notice(mut notice) = use_context();
    let Session(session) = use_context();
    use_future(move || async move {
        match api::get::<Settings>("settings").await {
            Ok(data) => settings.set(Some(data)),
            Err(error) => load_error.set(error),
        }
        loading.set(false);
    });
    use_effect(move || {
        let _ = category();
        query.set(String::new());
    });
    let data = settings();
    let mut categories = BTreeMap::<String, usize>::new();
    if let Some(data) = &data {
        for item in &data.items {
            *categories
                .entry(category_for(&item.path).into())
                .or_default() += 1;
        }
    }
    let searching = !query().trim().is_empty();
    let mut groups = BTreeMap::<String, Vec<Setting>>::new();
    if let Some(data) = &data {
        let needle = query().trim().to_lowercase();
        for item in &data.items {
            if (searching
                && format!("{} {}", item.label, item.path)
                    .to_lowercase()
                    .contains(&needle))
                || (!searching && category_for(&item.path) == category())
            {
                groups
                    .entry(
                        item.path
                            .rsplit_once('.')
                            .map(|(group, _)| group)
                            .unwrap_or("general")
                            .into(),
                    )
                    .or_default()
                    .push(item.clone());
            }
        }
    }
    let count: usize = groups.values().map(Vec::len).sum();
    rsx! {
        div { class: "settings-page",
            div { class: "settings-toolbar",
                div { class: "settings-search",
                    Search {}
                    Input {
                        id: "settings-search",
                        aria_label: "Search settings",
                        placeholder: "Search settings…",
                        value: query(),
                        disabled: data.is_none() || saving().is_some(),
                        oninput: move |event: FormEvent| query.set(event.value()),
                    }
                    if searching {
                        button {
                            r#type: "button",
                            disabled: saving().is_some(),
                            aria_label: "Clear search",
                            onclick: move |_| query.set(String::new()),
                            X {}
                        }
                    }
                }
                div { class: "settings-toolbar-actions",
                    Button {
                        variant: ButtonVariant::Ghost,
                        disabled: loading() || saving().is_some(),
                        onclick: move |_| {
                            loading.set(true);
                            load_error.set(String::new());
                            spawn(async move {
                                match api::get::<Settings>("settings").await {
                                    Ok(data) => {
                                        settings.set(Some(data));
                                        if !drafts.peek().is_empty() {
                                            notice
                                                .set(
                                                    Some(
                                                        crate::Feedback::info(
                                                            "Server values reloaded. Review your unsaved edits before saving.",
                                                        ),
                                                    ),
                                                );
                                        }
                                    }
                                    Err(error) => load_error.set(error),
                                }
                                loading.set(false);
                            });
                        },
                        if loading() {
                            Spinner {}
                        } else {
                            RefreshCw {}
                        }
                        "Reload"
                    }
                    Button {
                        disabled: loading() || saving().is_some() || drafts().is_empty() || data.is_none(),
                        onclick: move |_| {
                            let Some(current) = settings.peek().clone() else {
                                return;
                            };
                            let changes = drafts
                                .peek()
                                .iter()
                                .map(|(path, value)| SettingChange {
                                    path: path.clone(),
                                    value: value.clone(),
                                })
                                .collect();
                            let request = SettingsBatchUpdate {
                                revision: current.revision,
                                changes,
                            };
                            saving.set(Some("settings".into()));
                            load_error.set(String::new());
                            spawn(async move {
                                let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                match api::post::<_, String>("settings/batch", &request, &csrf).await {
                                    Ok(message) => {
                                        notice.set(Some(crate::Feedback::success(message)));
                                        match api::get::<Settings>("settings").await {
                                            Ok(fresh) => {
                                                drafts.write().clear();
                                                reset += 1;
                                                settings.set(Some(fresh));
                                            }
                                            Err(error) => {
                                                load_error
                                                    .set(
                                                        format!(
                                                            "Saved, but reload failed. Reload before saving again: {error}",
                                                        ),
                                                    )
                                            }
                                        }
                                    }
                                    Err(error) => load_error.set(error),
                                }
                                saving.set(None);
                            });
                        },
                        if saving().is_some() {
                            Spinner {}
                            "Saving…"
                        } else {
                            Save {}
                            "Save"
                        }
                    }
                }
            }
            if !load_error().is_empty() {
                ErrorState { message: load_error() }
            }
            if !drafts().is_empty() {
                div { class: "settings-draft-bar",
                    span { "{drafts().len()} unsaved setting(s)" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        disabled: saving().is_some(),
                        onclick: move |_| {
                            drafts.write().clear();
                            reset += 1;
                        },
                        RotateCcw {}
                        "Discard all"
                    }
                }
            }
            if data.is_none() && loading() {
                LoadingState { label: "Loading settings…" }
            } else if let Some(data) = data {
                div { class: "settings-layout",
                    nav {
                        class: "settings-categories",
                        aria_label: "Settings categories",
                        for (key, count) in &categories {
                            button {
                                r#type: "button",
                                disabled: saving().is_some(),
                                class: if !searching && category() == *key { "selected" } else { "" },
                                aria_pressed: !searching && category() == *key,
                                onclick: {
                                    let key = key.clone();
                                    move |_| {
                                        category.set(key.clone());
                                        query.set(String::new());
                                    }
                                },
                                CategoryIcon { category: key.clone() }
                                span { "{category_label(key)}" }
                                small { "{count}" }
                            }
                        }
                    }
                    div { class: "settings-mobile-category",
                        Label { html_for: "settings-category", "Category" }
                        FormSelect {
                            id: "settings-category",
                            value: category,
                            disabled: saving().is_some(),
                            options: categories
                                .keys()
                                .map(|key| FormSelectOption::new(key, category_label(key)))
                                .collect::<Vec<_>>(),
                        }
                    }
                    section {
                        class: "settings-content",
                        aria_label: "Settings editor",
                        div { class: "settings-category-heading",
                            h2 {
                                if searching {
                                    "Search results"
                                } else {
                                    "{category_label(&category())}"
                                }
                            }
                            p { class: "muted",
                                if searching {
                                    "{count} matching settings"
                                } else {
                                    "{category_caption(&category())}"
                                }
                            }
                        }
                        if groups.is_empty() {
                            EmptyState {
                                title: "No settings found",
                                description: "Try another name or configuration key.",
                            }
                        }
                        for (group, items) in groups {
                            Card { class: "settings-section",
                                div { class: "settings-section-heading",
                                    h3 {
                                        if searching {
                                            "{category_label(category_for(&group))} / "
                                        }
                                        "{section_label(&group)}"
                                    }
                                    span { "{items.len()} settings" }
                                }
                                for setting in items {
                                    SettingRow {
                                        key: "{setting.path}:{data.revision}:{reset}",
                                        setting,
                                        drafts,
                                        saving,
                                    }
                                }
                            }
                        }
                        p { class: "settings-config-note",
                            Lock {}
                            "Administrator credentials and TLS settings remain in the host configuration."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingRow(
    setting: Setting,
    mut drafts: Signal<BTreeMap<String, String>>,
    saving: Signal<Option<String>>,
) -> Element {
    let mut value = use_signal(|| {
        drafts
            .peek()
            .get(&setting.path)
            .cloned()
            .unwrap_or_else(|| setting.value.clone())
    });
    let mut touched = use_signal(|| drafts.peek().contains_key(&setting.path));
    let path = setting.path.clone();
    let original = setting.value.clone();
    let secret = setting.kind == "secret";
    let dirty = value() != setting.value || (secret && touched());
    use_effect(move || {
        let next = value();
        let changed = next != original || (secret && touched());
        if changed {
            if drafts.peek().get(&path) != Some(&next) {
                drafts.write().insert(path.clone(), next);
            }
        } else if drafts.peek().contains_key(&path) {
            drafts.write().remove(&path);
        }
    });
    let control = format!("setting-{}", setting.path.replace('.', "-"));
    let tip = use_hook(|| crate::hooks::use_random::use_random_id_for("setting-effect"));
    rsx! {
        div { class: "settings-row", "data-dirty": dirty.to_string(),
            div { class: "settings-row-copy",
                Label { html_for: control.clone(), "{setting.label}" }
                code { "{setting.path}" }
                Tooltip { class: "settings-effect",
                    span {
                        class: "settings-effect-label",
                        tabindex: "0",
                        aria_describedby: tip.clone(),
                        if setting.effect == "live" {
                            Check {}
                        } else {
                            Clock {}
                        }
                        "{effect_label(&setting.effect)}"
                    }
                    TooltipContent { id: tip.clone(), position: TooltipPosition::Top,
                        "{effect_help(&setting.effect)}"
                    }
                }
            }
            div { class: "settings-row-control",
                match setting.kind.as_str() {
                    "bool" => rsx! {
                        div { class: "settings-boolean",
                            span {
                                if value() == "true" {
                                    "Enabled"
                                } else {
                                    "Disabled"
                                }
                            }
                            Switch {
                                id: control.clone(),
                                aria_label: setting.label.clone(),
                                checked: value() == "true",
                                disabled: saving().is_some(),
                                on_checked_change: move |checked: bool| value.set(checked.to_string()),
                            }
                        }
                    },
                    "enum" => rsx! {
                        FormSelect {
                            id: control.clone(),
                            value,
                            disabled: saving().is_some(),
                            options: setting
                                .options
                                .iter()
                                .map(|option| FormSelectOption::new(option, option))
                                .collect::<Vec<_>>(),
                        }
                    },
                    "list" => rsx! {
                        Textarea {
                            id: control.clone(),
                            value: value(),
                            rows: 3,
                            disabled: saving().is_some(),
                            oninput: move |event: FormEvent| value.set(event.value()),
                        }
                        p { class: "settings-input-help", "Separate entries with commas." }
                    },
                    _ => rsx! {
                        Input {
                            id: control.clone(),
                            r#type: if secret { InputType::Password } else if setting.kind == "integer" { InputType::Number } else { InputType::Text },
                            value: value(),
                            autocomplete: if secret { "new-password" } else { "off" },
                            disabled: saving().is_some(),
                            placeholder: if secret { "Existing value hidden" } else { "" },
                            oninput: move |event: FormEvent| {
                                touched.set(true);
                                value.set(event.value());
                            },
                        }
                        if secret {
                            p { class: "settings-input-help", "Existing value hidden. Saving a blank value clears it." }
                        }
                        if let Some(unit) = unit_for(&setting.path) {
                            p { class: "settings-input-help", "{unit}" }
                        }
                    },
                }
                if dirty {
                    div { class: "settings-row-actions",
                        Button {
                            variant: ButtonVariant::Ghost,
                            disabled: saving().is_some(),
                            onclick: {
                                let original = setting.value.clone();
                                move |_| {
                                    value.set(original.clone());
                                    touched.set(false);
                                }
                            },
                            "Reset"
                        }
                    }
                }
            }
        }
    }
}
fn category_for(path: &str) -> &str {
    path.split('.').next().unwrap_or("general")
}
fn category_label(category: &str) -> &str {
    match category {
        "runtime" => "Runtime",
        "dns" => "DNS",
        "routing" => "Routing",
        "testing" => "Connection tests",
        "subscriptions" => "Subscriptions",
        "server" => "Server",
        "parser" => "Parser",
        "tui" => "Terminal UI",
        _ => category,
    }
}
fn category_caption(category: &str) -> &'static str {
    match category {
        "runtime" => "Proxy engines, listeners, and connection behavior.",
        "dns" => "Resolvers, DNS listeners, and fake IP behavior.",
        "routing" => "Direct connections, blocked destinations, and routing rules.",
        "testing" => "Timeouts and options for connection tests.",
        "subscriptions" => "How subscription sources are refreshed.",
        "server" => "HTTP server and subscription export options.",
        "parser" => "How imported configurations are interpreted.",
        "tui" => "Preferences for the terminal interface.",
        _ => "Configure this part of XRAT.",
    }
}
fn section_label(group: &str) -> String {
    if !group.contains('.') {
        return "General".into();
    }
    group
        .split('.')
        .skip(1)
        .map(|part| match part {
            "http" => "HTTP proxy".into(),
            "socks" => "SOCKS proxy".into(),
            "tun" => "TUN capture".into(),
            "dns" => "DNS".into(),
            "fakeip" => "Fake IP".into(),
            _ => {
                let text = part.replace('_', " ");
                let mut chars = text.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join(" / ")
}
fn effect_label(effect: &str) -> &'static str {
    match effect {
        "runtime restart" => "Runtime restart",
        "daemon restart" => "Daemon restart",
        _ => "Applies to new operations",
    }
}
fn effect_help(effect: &str) -> &'static str {
    match effect {
        "runtime restart" => "Restart the active proxy runtime to apply this change.",
        "daemon restart" => "Restart the XRAT daemon to apply this change.",
        _ => "Applies to subsequent operations without a restart.",
    }
}
fn unit_for(path: &str) -> Option<&'static str> {
    [
        ("_milliseconds", "Milliseconds"),
        ("_ms", "Milliseconds"),
        ("_seconds", "Seconds"),
        ("_secs", "Seconds"),
        ("_minutes", "Minutes"),
        ("_hours", "Hours"),
    ]
    .into_iter()
    .find_map(|(suffix, unit)| path.ends_with(suffix).then_some(unit))
}
#[component]
fn CategoryIcon(category: String) -> Element {
    match category.as_str() {
        "runtime" => rsx! {
            Cpu {}
        },
        "dns" => rsx! {
            Globe {}
        },
        "routing" => rsx! {
            Route {}
        },
        "testing" => rsx! {
            FlaskConical {}
        },
        "subscriptions" => rsx! {
            Rss {}
        },
        "server" => rsx! {
            Server {}
        },
        "parser" => rsx! {
            FileCode {}
        },
        _ => rsx! {
            Terminal {}
        },
    }
}
