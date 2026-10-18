use super::navigation::PageIcon;
use super::{
    avatar::{Avatar, AvatarFallback},
    button::{Button, ButtonSize, ButtonVariant},
    sheet::{Sheet, SheetContent, SheetDirection, SheetHeader, SheetTitle, SheetTrigger},
    sidenav::{
        Sidenav, SidenavCollapsible, SidenavContent, SidenavFooter, SidenavGroup,
        SidenavGroupLabel, SidenavHeader, SidenavMenu, SidenavMenuButton, SidenavMenuItem,
        SidenavTrigger, SidenavWrapper,
    },
    tooltip::{Tooltip, TooltipContent, TooltipPosition},
};
use dioxus::prelude::*;
use icons::{Menu, Moon, PanelLeftClose, Sun, UserRound};

#[component]
pub fn Sidebar(
    page: Signal<String>,
    dark: Signal<bool>,
    is_admin: bool,
    username: String,
) -> Element {
    let expanded = use_signal(|| {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("xrat-sidebar-collapsed").ok().flatten())
            .is_none_or(|value| value != "true")
    });
    use_effect(move || {
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item(
                "xrat-sidebar-collapsed",
                if expanded() { "false" } else { "true" },
            );
        }
    });
    rsx! {
        div { class: "desktop-sidebar",
            SidenavWrapper {
                class: "contents",
                controlled_open: expanded,
                style: "--sidenav-width:248px;--sidenav-width-icon:68px",
                Sidenav {
                    class: "sidebar",
                    data_collapsible: SidenavCollapsible::None,
                    SidebarBody {
                        page,
                        dark,
                        is_admin,
                        username: username.clone(),
                        compact: !expanded(),
                        on_expand: move |_| {
                            let mut expanded = expanded;
                            expanded.set(true);
                        },
                    }
                }
            }
        }
        div { class: "mobile-sidebar",
            Sheet {
                SheetTrigger { class: "mobile-menu-trigger",
                    Menu {}
                    span { "Navigation" }
                }
                SheetContent {
                    direction: SheetDirection::Left,
                    class: "w-[min(20rem,90vw)] p-3",
                    SheetHeader { class: "sr-only",
                        SheetTitle { "Workspace navigation" }
                    }
                    div { class: "mobile-sidebar-body",
                        SidebarBody {
                            page,
                            dark,
                            is_admin,
                            username,
                            mobile: true,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarBody(
    mut page: Signal<String>,
    mut dark: Signal<bool>,
    is_admin: bool,
    username: String,
    #[props(default)] compact: bool,
    #[props(default)] mobile: bool,
    #[props(optional)] on_expand: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        SidenavHeader { class: "sidebar-header",
            if compact {
                Tooltip { class: "m-0",
                    button {
                        class: "sidebar-brand sidebar-brand-toggle",
                        r#type: "button",
                        aria_label: "Expand sidebar",
                        aria_describedby: "tip-logo-expand",
                        onclick: move |event| {
                            if let Some(handler) = on_expand {
                                handler.call(event);
                            }
                        },
                        img {
                            src: "/xrat-logo.png",
                            alt: "XRAT",
                            width: "36",
                            height: "36",
                        }
                    }
                    TooltipContent {
                        id: "tip-logo-expand",
                        position: TooltipPosition::Right,
                        "Expand sidebar"
                    }
                }
            } else {
                div { class: "sidebar-brand",
                    img {
                        src: "/xrat-logo.png",
                        alt: "XRAT",
                        width: "36",
                        height: "36",
                    }
                    div { class: "sidebar-brand-label",
                        strong { "XRAT" }
                        span { "Connection control" }
                    }
                }
                if !mobile {
                    Tooltip { class: "m-0",
                        SidenavTrigger {
                            aria_label: "Collapse sidebar",
                            aria_describedby: "tip-sidebar-collapse",
                            PanelLeftClose {}
                        }
                        TooltipContent {
                            id: "tip-sidebar-collapse",
                            position: TooltipPosition::Right,
                            "Collapse sidebar"
                        }
                    }
                }
            }
        }
        nav {
            class: "sidebar-navigation",
            aria_label: "Main navigation",
            "data-sheet-navigation": mobile.to_string(),
            SidenavContent { class: "gap-4",
                SidenavGroup { class: "p-0",
                    SidenavGroupLabel { class: "sidebar-group-label", "Workspace" }
                    SidenavMenu {
                        for title in ["Overview", "Subscriptions", "Configs"] {
                            NavItem { title, page }
                        }
                        for title in ["Jobs", "XRAT logs", "Engine logs"] {
                            NavItem { title, page }
                        }
                    }
                }
                if is_admin {
                    SidenavGroup { class: "p-0",
                        SidenavGroupLabel { class: "sidebar-group-label", "Management" }
                        SidenavMenu {
                            NavItem { title: "Host administration", page }
                            NavItem { title: "Settings", page }
                            NavItem { title: "Users", page }
                        }
                    }
                }
                SidenavGroup { class: "p-0",
                    SidenavGroupLabel { class: "sidebar-group-label", "Tools" }
                    SidenavMenu {
                        for title in ["TCP scanner", "Parse configs", "GeoIP lookup"] {
                            NavItem { title, page }
                        }
                    }
                }
            }
        }
        SidenavFooter { class: "sidebar-footer",
            div { class: "sidebar-account-row",
                div {
                    class: "sidebar-account-link",
                    "data-sheet-navigation": mobile.to_string(),
                    Button {
                        variant: ButtonVariant::Ghost,
                        class: "sidebar-account",
                        aria_label: "Open Account",
                        onclick: move |_| page.set("Account".into()),
                        Avatar {
                            AvatarFallback { UserRound {} }
                        }
                        div { class: "sidebar-account-label",
                            strong { "{username}" }
                            span { "Account settings" }
                        }
                    }
                }
                Tooltip { class: "m-0",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        aria_label: if dark() { "Switch to light theme" } else { "Switch to dark theme" },
                        aria_describedby: if mobile { "tip-mobile-theme" } else { "tip-theme" },
                        onclick: move |_| {
                            dark.toggle();
                            if let Some(storage) = web_sys::window()
                                .and_then(|window| window.local_storage().ok().flatten())
                            {
                                let _ = storage
                                    .set_item("xrat-theme", if dark() { "dark" } else { "light" });
                            }
                        },
                        if dark() {
                            Sun {}
                        } else {
                            Moon {}
                        }
                    }
                    TooltipContent {
                        id: if mobile { "tip-mobile-theme" } else { "tip-theme" },
                        position: TooltipPosition::Right,
                        if dark() {
                            "Light theme"
                        } else {
                            "Dark theme"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavItem(title: &'static str, mut page: Signal<String>) -> Element {
    let label = if title == "Host administration" {
        "Host admin"
    } else {
        title
    };
    // Desktop and mobile navigation coexist, so tooltip IDs include the component scope.
    let tip = use_hook(|| crate::hooks::use_random::use_random_id_for("nav-tip"));
    rsx! {
        SidenavMenuItem {
            Tooltip { class: "sidebar-tooltip",
                SidenavMenuButton {
                    class: "sidebar-menu-button",
                    aria_label: title,
                    aria_describedby: tip.clone(),
                    aria_current: if page() == title { "page" } else { "false" },
                    onclick: move |_| page.set(title.into()),
                    PageIcon { title }
                    span { class: "sidebar-menu-label", "{label}" }
                }
                TooltipContent { id: tip, position: TooltipPosition::Right, "{title}" }
            }
        }
    }
}
