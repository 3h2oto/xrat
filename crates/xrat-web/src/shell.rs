use crate::{Session, pages, ui};
use dioxus::prelude::*;
use ui::sidebar::Sidebar;

#[component]
pub fn AppShell(page: Signal<String>, dark: Signal<bool>) -> Element {
    let Session(session) = use_context();
    let input = use_signal(String::new);
    let importing = use_signal(|| false);
    use_context_provider(|| crate::components::import::ImportState { input, importing });
    let title = if session().is_some_and(|me| me.user.must_change_password) {
        "Account".to_string()
    } else {
        page()
    };
    let main_class = if title == "Overview" {
        concat!(
            "overview-main [&_>_header]:mb-4 min-[1001px]:h700:[&_>_header]:mb-3 ",
            "min-[1001px]:h700:[&_>_header]:shrink-[0] ",
            "min-[1001px]:h700:min-h-0 ",
            "min-[1001px]:h700:flex-col ",
            "min-[1001px]:h700:pb-3 ",
            "min-[1001px]:h700:pt-4 ",
            "min-[1001px]:h700:flex ",
            "min-[1001px]:h700:overflow-y-auto ",
            "min-[1001px]:h700:[&_.page-title]:text-[26px] ",
            "min-[1001px]:h700:[&_.page-caption]:mt-1.5",
        )
    } else if matches!(title.as_str(), "Configs" | "Subscriptions") {
        concat!(
            "tables-main min-[801px]:h640:flex-col ",
            "min-[801px]:h640:min-h-0 ",
            "min-[801px]:h640:pb-3 ",
            "min-[801px]:h640:pt-5 ",
            "min-[801px]:h640:flex ",
            "min-[801px]:h640:overflow-y-auto ",
            "min-[801px]:h640:[&_>_header]:mb-4 ",
            "min-[801px]:h640:[&_>_header]:shrink-[0] ",
            "min-[801px]:h640:[&_>_.data-table-panel]:min-h-0 ",
            "min-[801px]:h640:[&_>_.data-table-panel]:pr-4 ",
            "min-[801px]:h640:[&_>_.data-table-panel]:pb-4 ",
            "min-[801px]:h640:[&_>_.data-table-panel]:pt-4 ",
            "min-[801px]:h640:[&_>_.data-table-panel]:flex-1 ",
            "min-[801px]:h640:[&_>_.data-table-panel]:pl-4 ",
            "min-[801px]:h640:[&_.data-table-panel_>_:not(.data-table-wrapper)]:shrink-[0] ",
            "min-[801px]:h640:[&_.data-page-heading]:gap-2.5 ",
            "min-[801px]:h640:[&_.data-page-heading]:mb-3.5 ",
            "min-[801px]:h640:[&_.data-toolbar]:mb-3 ",
            "min-[801px]:h640:[&_.data-filters]:mb-3 ",
            "min-[801px]:h640:[&_.data-selection-bar]:mb-3 ",
            "min-[801px]:h640:[&_.data-table-wrapper]:max-h-none ",
            "min-[801px]:h640:[&_.data-table-wrapper]:flex-1 ",
            "min-[801px]:h640:[&_.data-table-wrapper]:min-h-20 ",
            "min-[801px]:h640:[&_.data-pagination]:mt-3",
        )
    } else if title == "Settings" {
        "settings-main flex min-h-0 flex-col overflow-hidden [&_>_header]:shrink-0 [&_>_.settings-page]:flex-1 [&_>_.settings-page]:min-h-0"
    } else if matches!(title.as_str(), "XRAT logs" | "Engine logs") {
        concat!(
            "logs-main desktop:pt-5 desktop:flex desktop:pb-3 ",
            "desktop:flex-col desktop:[&_>_header]:mb-4 ",
            "desktop:[&_>_header]:shrink-[0] ",
            "desktop:[&_>_.log-page-panel]:flex-1 ",
            "desktop:[&_>_.log-page-panel]:min-h-0 ",
            "desktop:[&_.log-viewer]:flex desktop:[&_.log-viewer]:flex-col ",
            "desktop:[&_.log-viewer]:min-h-0 desktop:[&_.log-viewer]:flex-1 ",
            "desktop:[&_.log-viewer_>_:not(.log-console-shell)]:shrink-[0] ",
            "desktop:[&_.log-console-shell]:flex ",
            "desktop:[&_.log-console-shell]:flex-col ",
            "desktop:[&_.log-console-shell]:min-h-40 ",
            "desktop:[&_.log-console-shell]:flex-1 ",
            "desktop:[&_.log-console-heading]:shrink-[0] ",
            "desktop:[&_.log-output]:flex-1 ",
            "desktop:[&_.log-output]:max-h-none ",
            "desktop:[&_.log-output]:min-h-25 desktop:[&_.log-output]:h-auto",
        )
    } else {
        ""
    };
    let heading = rsx! {
        header {
            div {
                h1 { class: "page-title flex items-center gap-3 [&_>_svg]:size-6 [&_>_svg]:shrink-[0] [&_>_svg]:text-primary",
                    ui::navigation::PageIcon { title: title.clone() }
                    span { "{title}" }
                }
                p { class: "page-caption mt-2 mr-0 mb-0 ml-9 max-w-[65ch] text-muted-foreground text-[13px] leading-[1.6]",
                    {ui::navigation::page_caption(&title)}
                }
            }
        }
    };
    rsx! {
        if session().is_none() {
            pages::Login {}
        } else {
            if session().is_some_and(|me| me.user.role.can_operate() && !me.user.must_change_password) {
                crate::components::import::ImportController { section: page() }
            }
            Sidebar {
                page,
                dark,
                is_admin: session().is_some_and(|s| s.user.role.is_admin()),
                username: session().unwrap().user.username,
            }
            div { class: concat!(
                "workspace-shell flex flex-1 flex-col min-w-0 min-h-0 h-dvh mobile:h-auto [&_>_main]:w-full ",
                "[&_>_main]:min-h-0 [&_>_main]:overflow-y-auto min-[1001px]:h700:[&&:has(.overview-main)]:h-dvh ",
                "min-[1001px]:h700:[&&:has(.overview-main)]:min-h-0 ",
                "min-[1001px]:h700:[&&:has(.overview-main)_>_.app-footer]:shrink-0 ",
                "min-[801px]:h640:[&&:has(.tables-main)]:h-dvh min-[801px]:h640:[&&:has(.tables-main)]:min-h-0 ",
                "min-[801px]:h640:[&&:has(.tables-main)_>_.app-footer]:shrink-0 ",
                "[&&_>_main:has(.settings-page)]:[max-width:1448px] ",
                "[&&_>_main.settings-main]:overflow-hidden [&&:has(.settings-main)]:h-dvh ",
            ),
                main { class: main_class,
                    if title != "Settings" || session().is_some_and(|s| s.user.must_change_password) {
                        {heading.clone()}
                    }
                    if session().is_some_and(|s| s.user.must_change_password) {
                        pages::Account {}
                    } else {
                        match page().as_str() {
                            "Page not found" => rsx! {
                                crate::components::states::EmptyState {
                                    title: "Page not found",
                                    description: "Choose a page from the sidebar to continue.",
                                }
                            },
                            "Overview" => rsx! {
                                pages::OverviewPage {}
                            },
                            "Account" => rsx! {
                                pages::Account {}
                            },
                            "Users" => rsx! {
                                pages::Users {}
                            },
                            "Settings" => rsx! {
                                pages::SettingsPage { heading: heading.clone() }
                            },
                            "Tools" | "TCP scanner" | "Parse configs" | "GeoIP lookup" => rsx! {
                                pages::Tools { section: page() }
                            },
                            "XRAT logs" => rsx! {
                                pages::Workspace { section: "Logs" }
                            },
                            "Engine logs" => rsx! {
                                pages::EngineLogs {}
                            },
                            "Host administration" => rsx! {
                                pages::HostAdministration {}
                            },
                            "Jobs" => rsx! {
                                pages::Jobs {}
                            },
                            _ => rsx! {
                                pages::Workspace { section: page() }
                            },
                        }
                    }
                }
                ui::footer::Footer {}
            }
        }
    }
}
