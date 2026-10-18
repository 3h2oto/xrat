use crate::api;
use dioxus::prelude::*;
use icons::{ArrowUpRight, BookOpen, Github};
use xrat_web::types::VersionInfo;

#[component]
pub fn Footer() -> Element {
    let version = use_resource(|| async { api::get::<VersionInfo>("version").await });
    rsx! {
        footer { class: "app-footer",
            div { class: "footer-version", aria_live: "polite",
                match version.read().as_ref() {
                    Some(Ok(info)) => rsx! {
                        span { "XRAT v{info.version}" }
                        match info.update_available {
                            Some(true) => rsx! {
                                a {
                                    class: "footer-update",
                                    href: "https://github.com/mhyrzt/xrat/releases/latest",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "v{info.latest_version.as_deref().unwrap_or_default()} available"
                                    ArrowUpRight {}
                                }
                            },
                            Some(false) => rsx! {
                                span { class: "footer-status", "Up to date" }
                            },
                            None => rsx! {
                                span { class: "footer-status", "Update check unavailable" }
                            },
                        }
                    },
                    Some(Err(_)) => rsx! {
                        span { "XRAT" }
                        span { class: "footer-status", "Version check unavailable" }
                    },
                    None => rsx! {
                        span { "XRAT" }
                        span { class: "footer-status", "Checking version…" }
                    },
                }
            }
            nav { class: "footer-links", aria_label: "XRAT resources",
                FooterLink {
                    href: "https://github.com/mhyrzt/xrat",
                    label: "GitHub repository",
                    Github {}
                }
                FooterLink {
                    href: "https://mhyrzt.github.io/xrat",
                    label: "Documentation",
                    BookOpen {}
                }
            }
        }
    }
}

#[component]
fn FooterLink(href: &'static str, label: &'static str, children: Element) -> Element {
    let tip = use_hook(|| crate::hooks::use_random::use_random_id_for("footer-link"));
    rsx! {
        super::tooltip::Tooltip { class: "m-0",
            a {
                href,
                target: "_blank",
                rel: "noopener noreferrer",
                aria_label: label,
                aria_describedby: tip.clone(),
                {children}
            }
            super::tooltip::TooltipContent {
                id: tip.clone(),
                position: super::tooltip::TooltipPosition::Top,
                "{label}"
            }
        }
    }
}
