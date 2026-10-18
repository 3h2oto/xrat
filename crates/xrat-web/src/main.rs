use dioxus::prelude::*;
use xrat_web::types::*;
mod api;
mod hooks;
mod pages;
mod routing;
mod ui;
use ui::sidebar::Sidebar;

fn main() {
    dioxus::launch(App);
}
#[derive(Clone, Copy)]
pub struct Session(pub Signal<Option<LoginResponse>>);
#[derive(Clone, Copy)]
pub struct Notice(pub Signal<Option<Feedback>>);
pub use ui::feedback::Feedback;
#[component]
fn App() -> Element {
    let session = use_signal(|| None::<LoginResponse>);
    let notice = use_signal(|| None::<Feedback>);
    use_context_provider(|| Session(session));
    use_context_provider(|| Notice(notice));
    let mut restoring = use_signal(|| true);
    let mut session = session;
    let dark = use_signal(|| {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("xrat-theme").ok().flatten())
            .is_some_and(|v| v == "dark")
    });
    use_future(move || async move {
        if let Ok(me) = api::get::<LoginResponse>("me").await {
            session.set(Some(me));
        }
        restoring.set(false);
    });
    let page = routing::use_page_route();
    let title = if session().is_some_and(|me| me.user.must_change_password) {
        "Account".to_string()
    } else {
        page()
    };
    rsx! {
        div { class: if dark() { "dark app" } else { "app" },
            if restoring() {
                main { class: "login", "Restoring session…" }
            } else if session().is_none() {
                pages::Login {}
            } else {
                Sidebar {
                    page,
                    dark,
                    is_admin: session().is_some_and(|s| s.user.role.is_admin()),
                    username: session().unwrap().user.username,
                }
                div { class: "workspace-shell",
                    main { class: if title == "Overview" { "overview-main" } else if matches!(title.as_str(), "Configs" | "Subscriptions") { "tables-main" } else if matches!(title.as_str(), "XRAT logs" | "Engine logs") { "logs-main" } else { "" },
                        header {
                            div {
                                h1 { class: "page-title",
                                    ui::navigation::PageIcon { title: title.clone() }
                                    span { "{title}" }
                                }
                                p { class: "page-caption", {ui::navigation::page_caption(&title)} }
                            }
                        }
                        if session().is_some_and(|s| s.user.must_change_password) {
                            pages::Account {}
                        } else {
                            match page().as_str() {
                                "Page not found" => rsx! {
                                    ui::states::EmptyState {
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
                                    pages::SettingsPage {}
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
            ui::feedback::FeedbackView {}
        }
    }
}
