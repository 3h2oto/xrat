use super::*;

#[component]
pub(crate) fn SettingRow(
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
        div {
            class: concat!(
                "settings-row grid grid-cols-[minmax(0,_1fr)_minmax(200px,_0.85fr)] border-b border-solid ",
                "[&&[data-dirty=true]]:[background:color-mix(in_srgb,_var(--accent)_25%,_var(--panel))] ",
                "border-b-[color:color-mix(in_srgb,_var(--border)_25%,_transparent)] p-4.5 gap-6 narrow:pr-4 narrow:pb-4 ",
                "narrow:pt-4 narrow:pl-4 narrow:grid-cols-[minmax(0,_1fr)] ",
                "[@media(width>520px)_and_(width<=1050px)]:grid-cols-[minmax(0,_1fr)_minmax(160px,_0.85fr)] narrow:gap-3 ",
                "[@media(width>520px)_and_(width<=1050px)]:gap-3.5 [&:last-child]:border-b-0 ",
            ),
            "data-dirty": dirty.to_string(),
            div {
                class: concat!(
                    "settings-row-copy min-w-0 [&_>_label]:block [&_>_label]:text-[13px] [&_>_label]:font-medium ",
                    "[&_>_label]:leading-[1.5] [&_>_label]:mt-0 [&_>_label]:mb-1 [&_>_label]:mx-0 [&_>_code]:block ",
                    "[&_>_code]:wrap-anywhere [&_>_code]:text-muted-foreground [&_>_code]:text-[10px] ",
                    "[&_>_code]:leading-[1.6]",
                ),
                Label { html_for: control.clone(), "{setting.label}" }
                code { "{setting.path}" }
                Tooltip { class: "settings-effect block w-fit mt-[7px] mb-0 mx-0",
                    span {
                        class: "settings-effect-label inline-flex items-center text-[10px] text-muted-foreground rounded-[3px] gap-[5px] [&_svg]:w-[11px] [&_svg]:h-[11px]",
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
            div { class: concat!(
                "settings-row-control min-w-0 [align-self:start] [&_input]:h-8 [&_input]:text-[12px] [&_textarea]:min-h-18 ",
                "[&_textarea]:max-h-55 [&_textarea]:[field-sizing:fixed] [&_textarea]:font-mono [&_textarea]:text-[12px] ",
                "[&&_[data-name=SelectTrigger]]:[height:32px] [&&_[data-name=SelectTrigger]]:[font-size:12px] ",
            ),
                match setting.kind.as_str() {
                    "bool" => rsx! {
                        div { class: "settings-boolean flex items-center justify-end min-h-7 text-muted-foreground text-[11px] gap-2.5 narrow:justify-between",
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
                        p { class: "settings-input-help text-muted-foreground text-[10px] leading-[1.6] mt-1.5 mb-0 mx-0",
                            "Separate entries with commas."
                        }
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
                            p { class: "settings-input-help text-muted-foreground text-[10px] leading-[1.6] mt-1.5 mb-0 mx-0",
                                "Existing value hidden. Saving a blank value clears it."
                            }
                        }
                        if let Some(unit) = unit_for(&setting.path) {
                            p { class: "settings-input-help text-muted-foreground text-[10px] leading-[1.6] mt-1.5 mb-0 mx-0",
                                "{unit}"
                            }
                        }
                    },
                }
                if dirty {
                    div { class: "settings-row-actions flex justify-end items-center mt-2.5 gap-[5px]",
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
pub(crate) fn category_for(path: &str) -> &str {
    path.split('.').next().unwrap_or("general")
}
pub(crate) fn category_label(category: &str) -> &str {
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
pub(crate) fn category_caption(category: &str) -> &'static str {
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
pub(crate) fn section_label(group: &str) -> String {
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
pub(crate) fn effect_label(effect: &str) -> &'static str {
    match effect {
        "runtime restart" => "Runtime restart",
        "daemon restart" => "Daemon restart",
        _ => "Applies to new operations",
    }
}
pub(crate) fn effect_help(effect: &str) -> &'static str {
    match effect {
        "runtime restart" => "Restart the active proxy runtime to apply this change.",
        "daemon restart" => "Restart the XRAT daemon to apply this change.",
        _ => "Applies to subsequent operations without a restart.",
    }
}
pub(crate) fn unit_for(path: &str) -> Option<&'static str> {
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
