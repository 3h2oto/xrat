use crate::api;
use dioxus::prelude::*;
use icons::{ArrowUpRight, BookOpen, Github};
use xrat_web::types::VersionInfo;

#[component]
pub fn Footer() -> Element {
    let version = use_resource(|| async { api::get::<VersionInfo>("version").await });
    rsx! {
        footer {
            class: concat!(
                "app-footer sticky bottom-0 z-[var(--layer-footer)] shrink-0 flex items-center justify-between ",
                "mt-auto min-h-11 bg-background border-t border-solid ",
                "border-t-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] text-muted-foreground ",
                "text-[11px] py-2.5 px-11 gap-4 mobile:pr-4 ",
                "[@media(width>700px)_and_(width<=1100px)]:pr-6 mobile:pl-4 ",
                "[@media(width>700px)_and_(width<=1100px)]:pl-6 mobile:gap-2",
            ),
            div {
                class: "footer-version flex flex-wrap items-center min-w-0 wrap-anywhere gap-3 mobile:gap-1.5",
                aria_live: "polite",
                match version.read().as_ref() {
                    Some(Ok(info)) => rsx! {
                        span { "XRAT v{info.version}" }
                        match info.update_available {
                            Some(true) => rsx! {
                                a {
                                    class: "footer-update inline-flex items-center text-primary no-underline gap-[3px] [&:hover]:underline [&_svg]:w-3 [&_svg]:h-3",
                                    href: "https://github.com/mhyrzt/xrat/releases/latest",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "v{info.latest_version.as_deref().unwrap_or_default()} available"
                                    ArrowUpRight {}
                                }
                            },
                            Some(false) => rsx! {
                                span { class: "footer-status opacity-80", "Up to date" }
                            },
                            None => rsx! {
                                span { class: "footer-status opacity-80", "Update check unavailable" }
                            },
                        }
                    },
                    Some(Err(_)) => rsx! {
                        span { "XRAT" }
                        span { class: "footer-status opacity-80", "Version check unavailable" }
                    },
                    None => rsx! {
                        span { "XRAT" }
                        span { class: "footer-status opacity-80", "Checking version…" }
                    },
                }
            }
            nav {
                class: "footer-links flex items-center shrink-0 gap-1 [&_a]:flex [&_a]:items-center [&_a]:justify-center [&_a]:size-7 [&_a]:text-muted-foreground [&_a]:rounded-[5px] [&_a:hover]:bg-accent [&_a:hover]:text-foreground [&_svg]:size-[15px]",
                aria_label: "XRAT resources",
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
