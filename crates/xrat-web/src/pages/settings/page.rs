use super::components::*;
use crate::components::states::{EmptyState, ErrorState};
use crate::ui::{
    button::{Button, ButtonVariant},
    card::Card,
    form_select::{FormSelect, FormSelectOption},
    icon_button::IconButton,
    input::Input,
    label::Label,
    spinner::Spinner,
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{Lock, RefreshCw, RotateCcw, Save, Search, X};
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
        div { class: "settings-page flex min-h-0 w-full max-w-340 flex-col mx-auto [&_>_:not(.settings-layout)]:shrink-0",
            div { class: "settings-toolbar flex items-center justify-between mb-6 gap-4 narrow:flex-wrap narrow:gap-2",
                div {
                    class: concat!(
                        "settings-search min-w-0 flex items-center relative w-[min(100%,_430px)] [&_>_svg]:absolute ",
                        "[&_>_svg]:left-[11px] [&_>_svg]:w-[15px] [&_>_svg]:h-[15px] [&_>_svg]:text-muted-foreground ",
                        "[&_>_svg]:pointer-events-none [&_input]:h-8.5 [&_input]:text-[13px] [&_input]:px-8.5 ",
                        "[&_>_button]:absolute [&_>_button]:right-1.5 [&_>_button]:grid [&_>_button]:place-items-center ",
                        "[&_>_button]:w-6 [&_>_button]:h-6 [&_>_button]:border-0 [&_>_button]:rounded-[4px] ",
                        "[&_>_button]:bg-transparent [&_>_button]:text-muted-foreground [&_>_button:hover]:bg-accent ",
                        "[&_>_button_svg]:w-[13px] [&_>_button_svg]:h-[13px]",
                    ),
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
                div { class: "settings-toolbar-actions flex items-center shrink-0 gap-1.5 narrow:ml-auto",
                    IconButton {
                        label: if loading() { "Reloading settings" } else { "Reload settings" },
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
                    }
                    IconButton {
                        label: if saving().is_some() { "Saving settings" } else { "Save settings" },
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
                        } else {
                            Save {}
                        }
                    }
                }
            }
            if !load_error().is_empty() {
                ErrorState { message: load_error() }
            }
            if !drafts().is_empty() {
                div { class: "settings-draft-bar flex justify-between items-center flex-wrap mb-5 bg-accent rounded-[6px] text-[12px] py-2 px-3 gap-3",
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
                super::suspend::Suspend {}
            } else if let Some(data) = data {
                div { class: "settings-layout flex min-h-0 flex-1 flex-col gap-6 [@media(width<=1050px)]:gap-5 [&_>_nav]:shrink-0 [&_>_.settings-mobile-category]:shrink-0",
                    nav {
                        class: concat!(
                            "settings-categories flex flex-wrap justify-center pb-4 border-b border-solid ",
                            "border-b-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] gap-[5px] ",
                            "[@media(width<=800px)]:hidden [&_button]:flex [&_button]:items-center [&_button]:border-0 ",
                            "[&_button]:rounded-[6px] [&_button]:text-muted-foreground [&_button]:bg-transparent ",
                            "[&_button]:text-[12px] [&_button]:text-left [&_button]:py-[9px] [&_button]:px-2.5 ",
                            "[&_button]:gap-2.5 [&_button:hover]:text-foreground [&_button:hover]:bg-accent ",
                            "[&_button.selected]:text-foreground [&_button.selected]:bg-accent ",
                            "[&_button.selected]:font-semibold [&_svg]:size-[15px] [&_svg]:shrink-[0] [&_small]:ml-1.5 ",
                            "[&_small]:text-[10px] [&_small]:tabular-nums [&_small]:text-muted-foreground",
                        ),
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
                    div { class: "settings-mobile-category hidden [@media(width<=800px)]:block [@media(width<=800px)]:[&_label]:mb-1.5",
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
                        class: "settings-content min-h-0 min-w-0 flex-1 overflow-y-auto overscroll-contain pr-2 [scrollbar-gutter:stable]",
                        aria_label: "Settings editor",
                        div { class: "settings-category-heading mb-5 [&_h2]:text-[19px] [&_h2]:m-0 [&_p]:mt-[5px] [&_p]:mb-0 [&_p]:mx-0",
                            h2 {
                                if searching {
                                    "Search results"
                                } else {
                                    "{category_label(&category())}"
                                }
                            }
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
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
                            Card { class: "settings-section mb-5 border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] rounded-[9px] shadow-none p-0 gap-0",
                                div {
                                    class: concat!(
                                        "settings-section-heading flex justify-between items-center border-b border-solid ",
                                        "border-b-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] py-3.5 px-4.5 gap-3 ",
                                        "narrow:pr-4 narrow:pl-4 [&_h3]:text-[13px] ",
                                        "[&_h3]:font-semibold [&_h3]:m-0 [&_>_span]:text-[10px] [&_>_span]:text-muted-foreground",
                                    ),
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
                        p { class: "settings-config-note flex items-start text-muted-foreground text-[11px] leading-[1.6] mt-6 mb-0 mx-0 gap-2 [&_svg]:size-[13px] [&_svg]:shrink-[0] [&_svg]:mt-0.5",
                            Lock {}
                            "Administrator credentials and TLS settings remain in the host configuration."
                        }
                    }
                }
            }
        }
    }
}
