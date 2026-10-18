use crate::ui::{
    button::{Button, ButtonVariant},
    card::Card,
    field::Field,
    input::{Input, InputType},
    label::Label,
    spinner::Spinner,
    states::{ErrorState, LoadingLayout, LoadingState},
    textarea::Textarea,
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{ArrowRight, CircleCheck, CircleX, Copy, Download, FileCode, Globe, Radar, RotateCcw};
use xrat_web::types::{Action, Job};

#[component]
pub fn Tools(section: String) -> Element {
    let Session(session) = use_context();
    if !session().is_some_and(|session| session.user.role.can_operate()) {
        return rsx! {
            p { class: "empty text-muted-foreground leading-[1.8] text-[14px] py-9 px-3",
                "Tools require an operator or administrator account."
            }
        };
    }
    rsx! {
        for tool in [section] {
            ToolEditor { key: "{tool}", section: tool }
        }
    }
}

#[component]
fn ToolEditor(section: String) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut input = use_signal(String::new);
    let mut port = use_signal(|| "443".to_string());
    let mut submitting = use_signal(|| false);
    let mut job = use_signal(|| None::<Job>);
    let mut error = use_signal(String::new);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(2000).await;
            let Some(current) = job() else { continue };
            if !is_active(&current) {
                continue;
            }
            match api::get::<Vec<Job>>("jobs").await {
                Ok(jobs) => {
                    if let Some(updated) = jobs.into_iter().find(|updated| updated.id == current.id)
                    {
                        job.set(Some(updated));
                        error.set(String::new());
                    } else {
                        error.set(
                            "This job is no longer in recent history. Check Jobs for details."
                                .into(),
                        );
                    }
                }
                Err(message) => error.set(format!("Progress could not be refreshed: {message}")),
            }
        }
    });
    let scanner = section == "TCP scanner";
    let parser = section == "Parse configs";
    let busy = submitting() || job().as_ref().is_some_and(is_active);
    let scan = if scanner {
        scan_input(&input(), &port())
    } else {
        Ok((Vec::new(), 443))
    };
    let valid = !input().trim().is_empty() && scan.is_ok();
    let submit_label = if scanner {
        "Scan addresses"
    } else if parser {
        "Parse links"
    } else {
        "Look up"
    };
    let input_label = if scanner {
        "IP addresses"
    } else if parser {
        "Configuration links"
    } else {
        "IP address or hostname"
    };
    let result_id = use_hook(|| crate::hooks::use_random::use_random_id_for("tool-result"));
    let action_section = section.clone();
    rsx! {
        div { class: "tool-page grid grid-cols-[minmax(0,_1.15fr)_minmax(0,_1fr)] max-w-320 items-start mx-auto gap-7 [@media(width<=850px)]:grid-cols-[minmax(0,_1fr)] [@media(width<=850px)]:gap-2 [&_.error]:wrap-anywhere",
            Card { class: "tool-editor rounded-[9px] shadow-none border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] p-5.5 narrow:pr-4 narrow:pb-4 narrow:pt-4 narrow:pl-4 [&_label]:text-[12px]",
                form {
                    onsubmit: move |event| {
                        event.prevent_default();
                        if busy || !valid {
                            return;
                        }
                        let action = match action_section.as_str() {
                            "TCP scanner" => {
                                let Ok((ips, port)) = scan_input(&input(), &port()) else {
                                    return;
                                };
                                Action::Scan {
                                    ips,
                                    port,
                                    timeout_ms: 4000,
                                }
                            }
                            "Parse configs" => Action::Parse { input: input() },
                            _ => {
                                Action::GeoIpLookup {
                                    address: input().trim().into(),
                                }
                            }
                        };
                        submitting.set(true);
                        error.set(String::new());
                        job.set(None);
                        spawn(async move {
                            let csrf = session().map(|session| session.csrf_token).unwrap_or_default();
                            match api::post::<_, Job>("actions", &action, &csrf).await {
                                Ok(started) => {
                                    notice.set(Some(crate::Feedback::job_started(&started.kind)));
                                    job.set(Some(started));
                                }
                                Err(message) => error.set(message),
                            }
                            submitting.set(false);
                        });
                    },
                    div { class: "tool-section-heading mb-5.5 [&_h2]:text-[16px] [&_h2]:font-semibold [&_h2]:m-0 [&_p]:mt-1.5 [&_p]:mb-0 [&_p]:max-w-[48ch] [&_p]:text-muted-foreground [&_p]:text-[12px] [&_p]:leading-[1.7] [&_p]:mx-0",
                        h2 {
                            if scanner {
                                "Scan targets"
                            } else if parser {
                                "Check your links"
                            } else {
                                "Find an address"
                            }
                        }
                        p {
                            if scanner {
                                "Check TCP reachability from the XRAT host."
                            } else if parser {
                                "Recognize protocols without importing or changing your configurations."
                            } else {
                                "Resolve an address and look up its country, city, and network."
                            }
                        }
                    }
                    Field {
                        Label { html_for: "tool-input", "{input_label}" }
                        if scanner || parser {
                            Textarea {
                                id: "tool-input",
                                class: "tool-text-input min-h-42.5 max-h-[45dvh] resize-y font-mono text-[12px] leading-[1.7]",
                                rows: 7,
                                disabled: busy,
                                value: input(),
                                placeholder: if scanner { "1.1.1.1\n2606:4700:4700::1111" } else { "Paste configuration links here…" },
                                oninput: move |event: FormEvent| input.set(event.value()),
                            }
                        } else {
                            Input {
                                id: "tool-input",
                                value: input(),
                                disabled: busy,
                                placeholder: "1.1.1.1 or example.com",
                                oninput: move |event: FormEvent| input.set(event.value()),
                            }
                        }
                        p {
                            id: "tool-input-help",
                            class: "tool-input-help text-muted-foreground text-[11px] leading-[1.65] mt-1.5 mb-0 mx-0",
                            if scanner {
                                "One IPv4 or IPv6 address per line, or separated by commas. Up to 4,096 addresses."
                            } else if parser {
                                "Paste one or more links. Nothing is saved to Configs."
                            } else {
                                "Uses the GeoIP providers configured on this host."
                            }
                        }
                    }
                    if scanner {
                        div { class: "tool-scan-options flex items-end mt-4.5 gap-4 [&_>_div]:w-27.5 [&_>_div]:shrink-[0] [&_p]:mt-0 [&_p]:mb-2 [&_p]:mx-0",
                            Field {
                                Label { html_for: "scan-port", "Port" }
                                Input {
                                    id: "scan-port",
                                    r#type: InputType::Number,
                                    min: "1",
                                    max: "65535",
                                    disabled: busy,
                                    value: port(),
                                    oninput: move |event: FormEvent| port.set(event.value()),
                                }
                            }
                            p { class: "tool-input-help text-muted-foreground text-[11px] leading-[1.65] mt-1.5 mb-0 mx-0",
                                "4 second timeout per address"
                            }
                        }
                        if !input().trim().is_empty() {
                            if let Err(message) = &scan {
                                p {
                                    class: "error text-destructive text-[14px]",
                                    role: "alert",
                                    "{message}"
                                }
                            }
                        }
                    }
                    div { class: "tool-actions flex items-center flex-wrap mt-6 pt-4 border-t border-solid border-t-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] gap-[7px]",
                        Button { button_type: "submit", disabled: busy || !valid,
                            if busy {
                                Spinner {}
                            } else if scanner {
                                Radar {}
                            } else if parser {
                                FileCode {}
                            } else {
                                Globe {}
                            }
                            if busy {
                                "Working…"
                            } else {
                                "{submit_label}"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            disabled: busy || input().is_empty(),
                            onclick: move |_| {
                                input.set(String::new());
                                port.set("443".into());
                                job.set(None);
                                error.set(String::new());
                            },
                            RotateCcw {}
                            "Clear"
                        }
                        if parser {
                            a {
                                class: concat!(
                                    "tool-export inline-flex items-center text-muted-foreground text-[11px] ml-auto gap-[5px] ",
                                    "narrow:basis-full narrow:mr-0 narrow:ml-0 ",
                                    "narrow:mb-0 narrow:mt-2 [&:hover]:text-foreground ",
                                    "[&_svg]:w-[13px] [&_svg]:h-[13px]",
                                ),
                                href: "/api/v1/export",
                                download: "xrat-subscription.txt",
                                Download {}
                                "Export enabled configs"
                            }
                        }
                    }
                }
            }
            section {
                class: "tool-results min-w-0 py-4.5 px-0 [@media(width<=850px)]:pr-1 [@media(width<=850px)]:pl-1",
                aria_label: "Tool result",
                aria_live: "polite",
                div { class: "tool-result-heading [&_h2]:text-[16px] [&_h2]:font-semibold [&_h2]:m-0 flex items-center justify-between mb-4.5 gap-3",
                    h2 { "Result" }
                    if let Some(current) = job() {
                        ToolJobStatus { state: current.state }
                    }
                }
                if !error().is_empty() {
                    ErrorState { message: error() }
                }
                if let Some(current) = job() {
                    if let Some(output) = &current.result {
                        pre {
                            id: result_id.clone(),
                            class: "tool-output max-h-[45dvh] overflow-auto wrap-anywhere whitespace-pre-wrap rounded-[6px] bg-accent text-[12px] leading-[1.85] tabular-nums p-4",
                            tabindex: "0",
                            "{output}"
                        }
                    } else if is_active(&current) {
                        LoadingState {
                            label: "Waiting for the tool result",
                            layout: LoadingLayout::Summary,
                        }
                    } else {
                        p { class: "tool-result-placeholder flex items-center text-muted-foreground text-[12px] leading-[1.8] py-3.5 px-0 gap-[7px]",
                            "This job finished without additional output."
                        }
                    }
                    div { class: "tool-result-actions [&_a:hover]:text-foreground [&_svg]:w-[13px] [&_svg]:h-[13px] flex items-center justify-between mt-3.5 gap-2.5 [&_a]:inline-flex [&_a]:items-center [&_a]:text-muted-foreground [&_a]:text-[11px] [&_a]:gap-[5px]",
                        a { href: "/ui/jobs",
                            "View in Jobs"
                            ArrowRight {}
                        }
                        if current.result.is_some() {
                            button {
                                r#type: "button",
                                class: concat!(
                                    "console-copy inline-flex h-7.5 text-[12px] font-medium items-center border border-solid ",
                                    "border-border rounded-[0.4rem] bg-transparent cursor-pointer text-muted-foreground ",
                                    "border-transparent py-0 px-[0.6rem] gap-[0.4rem] [&_svg]:size-4 [&:hover]:text-foreground ",
                                    "[&:hover]:bg-accent [&:disabled]:opacity-40 [&:disabled]:cursor-default",
                                ),
                                "data-copy-textarea": result_id.clone(),
                                Copy {}
                                span { "Copy" }
                            }
                        }
                    }
                } else if submitting() {
                    LoadingState { label: "Starting job", layout: LoadingLayout::Summary }
                } else if error().is_empty() {
                    div {
                        class: concat!(
                            "tool-result-empty text-muted-foreground py-9.5 px-3 [@media(width<=850px)]:pr-2 ",
                            "[@media(width<=850px)]:pb-5 [@media(width<=850px)]:pt-5 [@media(width<=850px)]:pl-2 ",
                            "[&_>_svg]:w-[25px] [&_>_svg]:h-[25px] [&_>_svg]:opacity-65 [&_>_svg]:mb-4 [&_p]:text-[14px] ",
                            "[&_p]:font-medium [&_p]:text-foreground [&_p]:mt-0 [&_p]:mb-[7px] [&_p]:mx-0 [&_span]:block ",
                            "[&_span]:max-w-[39ch] [&_span]:text-[12px] [&_span]:leading-[1.8]",
                        ),
                        if scanner {
                            Radar {}
                        } else if parser {
                            FileCode {}
                        } else {
                            Globe {}
                        }
                        p {
                            if scanner {
                                "Ready when you are"
                            } else if parser {
                                "Your links, decoded"
                            } else {
                                "A little context for any address"
                            }
                        }
                        span {
                            if scanner {
                                "Add targets to see scan progress here. Results are stored in scanner history; you can cancel a running scan in Jobs."
                            } else if parser {
                                "Paste your links to see how many configurations XRAT recognizes and which protocols they use."
                            } else {
                                "Enter an IP address or hostname to see its GeoIP result here."
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ToolJobStatus(state: String) -> Element {
    let active = matches!(state.as_str(), "queued" | "running");
    let label = match state.as_str() {
        "queued" | "running" => "In progress",
        "succeeded" => "Complete",
        "cancelled" => "Cancelled",
        "interrupted" => "Interrupted",
        _ => "Failed",
    };
    rsx! {
        span {
            class: concat!(
                "tool-job-state inline-flex items-center text-[11px] text-muted-foreground gap-1.5 [&_svg]:w-3.5 ",
                "[&_svg]:h-3.5 [&&[data-state=failed]]:text-destructive [&&[data-state=interrupted]]:text-destructive ",
                "[&&[data-state=succeeded]]:text-foreground ",
            ),
            "data-state": state.clone(),
            if active {
                Spinner {}
            } else if state == "succeeded" {
                CircleCheck {}
            } else {
                CircleX {}
            }
            "{label}"
        }
    }
}

fn is_active(job: &Job) -> bool {
    matches!(job.state.as_str(), "queued" | "running")
}

fn scan_input(input: &str, port: &str) -> Result<(Vec<String>, u16), String> {
    let port = port
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| "Enter a port between 1 and 65,535.".to_string())?;
    let targets = input
        .split([',', '\n'])
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .collect::<Vec<_>>();
    if targets.is_empty() || targets.len() > 4096 {
        return Err("Enter between 1 and 4,096 IP addresses.".into());
    }
    let ips = targets.into_iter().map(|target| target.parse::<std::net::IpAddr>().map(|ip| ip.to_string()).map_err(|_| format!("{target} is not a valid IP address. Hostnames are not supported by the TCP scanner."))).collect::<Result<Vec<_>, _>>()?;
    Ok((ips, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_status_renders_labels_as_text() {
        for (state, expected) in [
            ("cancelled", "Cancelled"),
            ("interrupted", "Interrupted"),
            ("failed", "Failed"),
        ] {
            let mut dom = VirtualDom::new_with_props(
                ToolJobStatus,
                ToolJobStatusProps {
                    state: state.into(),
                },
            );
            let mutations = dom.rebuild_to_vec();
            assert!(mutations.edits.iter().any(|edit| matches!(edit, dioxus::core::Mutation::CreateTextNode { value, .. } if value == expected)));
        }
    }

    #[test]
    fn scan_accepts_mixed_ip_families_and_separators() {
        let (ips, port) = scan_input(" 1.1.1.1,\n2606:4700:4700::1111\n", "443").unwrap();
        assert_eq!(ips, ["1.1.1.1", "2606:4700:4700::1111"]);
        assert_eq!(port, 443);
    }

    #[test]
    fn scan_rejects_hostnames_and_invalid_ports_or_target_counts() {
        assert!(scan_input("example.com", "443").is_err());
        for port in ["0", "65536", "", "-1"] {
            assert!(scan_input("1.1.1.1", port).is_err());
        }
        assert!(scan_input(",\n", "443").is_err());
        assert!(scan_input(&"1.1.1.1\n".repeat(4097), "443").is_err());
    }
}
