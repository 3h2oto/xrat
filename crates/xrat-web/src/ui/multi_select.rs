use super::{
    button::{Button, ButtonVariant},
    dropdown_menu::*,
    input::Input,
    label::Label,
};
use dioxus::prelude::*;
use icons::{Check, ChevronDown};
use std::collections::BTreeSet;
use xrat_web::configs::sharing::filter_values;

#[component]
pub fn MultiSelect(
    id: String,
    label: String,
    value: String,
    options: Vec<(String, String)>,
    on_change: EventHandler<String>,
) -> Element {
    let mut query = use_signal(String::new);
    let selected = filter_values(&value)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let display = if selected.len() > 1 {
        format!("{} selected", selected.len())
    } else {
        options
            .iter()
            .find(|(key, _)| *key == value)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| value.clone())
    };
    let needle = query().trim().to_lowercase();
    let searchable = options.len() > 8;
    let shown = options
        .into_iter()
        .filter(|(key, label)| key.is_empty() || label.to_lowercase().contains(&needle))
        .collect::<Vec<_>>();
    rsx! {
        DropdownMenu {
            DropdownMenuTrigger { as_child: true,
                Button {
                    id: id.clone(),
                    aria_label: label.clone(),
                    aria_haspopup: "menu",
                    variant: ButtonVariant::Outline,
                    class: "w-full h-7.5 justify-between px-2 text-[11px] [&_svg]:size-3",
                    onclick: move |_| query.set(String::new()),
                    span { class: "min-w-0 truncate", "{display}" }
                    ChevronDown {}
                }
            }
            DropdownMenuContent { class: "w-64 max-w-[calc(100vw-2rem)]",
                DropdownMenuLabel { "{label}" }
                if searchable {
                    div { class: "px-1 pb-2",
                        Label { class: "sr-only", html_for: format!("{id}-search"), "Search {label}" }
                        Input { id: format!("{id}-search"), placeholder: "Search…", value: query(), oninput: move |event: FormEvent| query.set(event.value()) }
                    }
                }
                div { class: "max-h-64 overflow-y-auto overscroll-contain",
                    for (key, option_label) in shown.iter() {
                        button {
                            key: "{key}",
                            r#type: "button",
                            role: "menuitemcheckbox",
                            tabindex: "-1",
                            aria_checked: if key.is_empty() { selected.is_empty() } else { selected.contains(key) },
                            class: "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-xs hover:bg-accent focus-visible:bg-accent",
                            onclick: {
                                let key = key.clone();
                                let selected = selected.clone();
                                move |_| {
                                    let mut next = selected.clone();
                                    if key.is_empty() {
                                        next.clear();
                                    } else if !next.remove(&key) {
                                        next.insert(key.clone());
                                    }
                                    on_change.call(next.into_iter().collect::<Vec<_>>().join(","));
                                }
                            },
                            span { class: "size-3.5 shrink-0 rounded-sm border border-border [&_svg]:size-3",
                                if if key.is_empty() { selected.is_empty() } else { selected.contains(key) } { Check {} }
                            }
                            "{option_label}"
                        }
                    }
                    if shown.iter().all(|(key, _)| key.is_empty()) && !needle.is_empty() {
                        p { class: "px-2 py-3 text-xs text-muted-foreground", "No matches" }
                    }
                }
            }
        }
    }
}
