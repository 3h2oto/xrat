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
        div { class: "desktop-sidebar shrink-0 relative z-[var(--layer-sidebar)] mobile:hidden",
            SidenavWrapper {
                class: "contents",
                controlled_open: expanded,
                style: "--sidenav-width:248px;--sidenav-width-icon:68px",
                Sidenav {
                    class: concat!(
                        "sidebar sticky top-0 flex flex-col flex-[0_0_248px] w-62 h-dvh bg-card border-r border-solid ",
                        "[&&]:[transition:width_0.18s_ease,_flex-basis_0.18s_ease] desktop:[&&[data-collapsed=true]]:[width:68px] ",
                        "desktop:[&&[data-collapsed=true]]:[flex-basis:68px] desktop:[&&[data-collapsed=true]]:[padding:12px_8px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-header]:flex-col ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-header]:[gap:12px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-header]:[padding-bottom:12px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-brand]:flex-[0] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-brand]:justify-center ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-brand-label]:hidden ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-menu-label]:hidden ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-label]:hidden ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-group-label]:[height:1px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-group-label]:[font-size:0] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-group-label]:p-0 ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-group-label]:[margin:8px_4px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-group-label]:border-t-0 ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-footer]:border-t-0 ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-footer]:w-full ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-navigation]:p-0 ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-navigation]:overflow-x-hidden ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-menu-button]:justify-center ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-menu-button]:[padding:10px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-row]:flex-col ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-row]:[gap:8px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-row]:items-center ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account]:[padding:4px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account]:flex-none ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account]:[width:40px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account]:[height:40px] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account]:gap-0 ",
                        "mobile:[&&[data-mobile-open=true]_.sidebar-navigation]:flex ",
                        "mobile:[&&[data-mobile-open=true]_.sidebar-navigation]:[margin-top:12px] ",
                        "mobile:[&&[data-mobile-open=true]_.sidebar-navigation]:[max-height:55dvh] ",
                        "mobile:[&&[data-mobile-open=true]_.sidebar-footer]:block ",
                        "[&&[data-collapsed=true]_.sidebar-account]:justify-center ",
                        "[&&_[data-name=SidenavHeader]]:[flex-direction:row] [&&_[data-name=SidenavContent]]:overflow-visible ",
                        "[&&_[data-name=SidenavContent]]:min-w-0 [&&_[data-name=SidenavContent]]:w-full ",
                        "[&&_[data-name=SidenavMenuButton][aria-current=page]]:bg-accent ",
                        "[&&_[data-name=SidenavMenuButton][aria-current=page]]:text-foreground ",
                        "[&&_[data-name=SidenavMenuButton][aria-current=page]]:[font-weight:650] ",
                        "[&&_[data-name=SidenavMenuButton][aria-current=page]_svg]:text-primary ",
                        "[&&_[data-name=SidenavFooter]]:[padding:12px_0_0] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-menu]:flex ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-menu]:justify-center ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-menu]:w-full ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-menu_[data-name=DropdownMenu]]:justify-center ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-menu_[data-name=DropdownMenu]]:w-full ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-menu_[data-name=DropdownMenu]]:min-w-0 ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-row_>_[data-name=Tooltip]]:flex ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-row_>_[data-name=Tooltip]]:justify-center ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-row_>_[data-name=Tooltip]]:m-0 ",
                        "[&&[data-collapsed=true]_.sidebar-account-link]:w-full ",
                        "[&&[data-collapsed=true]_.sidebar-account-link]:flex-[0] ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-link]:flex ",
                        "desktop:[&&[data-collapsed=true]_.sidebar-account-link]:justify-center border-r-border p-3 mobile:w-full ",
                        "mobile:pr-4 mobile:flex-none mobile:border-r-0 mobile:border-b mobile:border-solid mobile:border-b-border ",
                        "mobile:pl-4 mobile:h-auto mobile:relative ",
                    ),
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
        div {
            class: concat!(
                "mobile-sidebar hidden mobile:min-h-14 mobile:justify-between ",
                "mobile:pr-3 mobile:border-b ",
                "mobile:border-solid mobile:border-b-border ",
                "mobile:pb-1 mobile:pt-1 mobile:bg-card ",
                "mobile:shrink-[0] mobile:pl-4 ",
                "mobile:items-center mobile:gap-3 mobile:flex",
            ),
            div {
                class: concat!(
                    "mobile-nav-heading mobile:min-w-0 mobile:items-center ",
                    "mobile:gap-2.5 mobile:flex ",
                    "mobile:[&_img]:object-contain mobile:[&_img]:shrink-[0] ",
                    "mobile:[&_strong]:tracking-[0.04em] ",
                    "mobile:[&_strong]:text-[13px] mobile:[&_strong]:font-semibold",
                ),
                img {
                    src: "/media/xrat-logo.png",
                    alt: "XRAT",
                    width: "28",
                    height: "28",
                }
                strong { "XRAT" }
                span {
                    class: "mobile-nav-divider mobile:text-border",
                    aria_hidden: "true",
                    "/"
                }
                span { class: "mobile-nav-page mobile:[text-overflow:ellipsis] mobile:whitespace-nowrap mobile:text-[12px] mobile:text-muted-foreground mobile:overflow-hidden",
                    "{page}"
                }
            }
            Sheet {
                SheetTrigger {
                    class: concat!(
                        "mobile-menu-trigger mobile:w-11 mobile:shadow-none ",
                        "mobile:pr-0 mobile:bg-transparent mobile:pb-0 ",
                        "mobile:pt-0 mobile:pl-0 mobile:text-foreground ",
                        "mobile:h-11 mobile:border-0 mobile:[&_svg]:w-5 ",
                        "mobile:[&_svg]:h-5 ",
                        "mobile:[&:focus-visible]:[outline-offset:-4px] ",
                        "mobile:[&:focus-visible]:[outline:2px_solid_var(--ring)] ",
                        "mobile:[&:active]:bg-accent",
                    ),
                    Menu {}
                    span { class: "sr-only", "Open navigation" }
                }
                SheetContent {
                    direction: SheetDirection::Left,
                    class: "mobile-navigation-sheet w-[min(20rem,90vw)] p-3 h-dvh overflow-hidden flex flex-col",
                    SheetHeader { class: "sr-only",
                        SheetTitle { "Workspace navigation" }
                    }
                    div { class: "mobile-sidebar-body flex-1 min-h-0 flex flex-col [&_.sidebar-header]:flex-row [&_.sidebar-header]:pr-8 [&_.sidebar-navigation]:flex [&_.sidebar-footer]:block",
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
    super::sheet::use_sheet_navigation(page, mobile);
    rsx! {
        SidenavHeader { class: "sidebar-header flex items-center pt-1 pb-4 px-0.5 gap-2 mobile:pr-0 mobile:pb-0 mobile:pt-0 mobile:pl-0",
            if compact {
                Tooltip { class: "m-0",
                    button {
                        class: "sidebar-brand sidebar-brand-toggle flex items-center min-w-0 flex-1 no-underline text-foreground border-0 bg-transparent rounded-[8px] p-0 gap-2.5 [&_img]:object-contain [&_img]:shrink-[0] [&:hover]:bg-accent",
                        r#type: "button",
                        aria_label: "Expand sidebar",
                        aria_describedby: "tip-logo-expand",
                        onclick: move |event| {
                            if let Some(handler) = on_expand {
                                handler.call(event);
                            }
                        },
                        img {
                            src: "/media/xrat-logo.png",
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
                div { class: "sidebar-brand flex items-center min-w-0 flex-1 no-underline text-foreground gap-2.5 [&_img]:object-contain [&_img]:shrink-[0]",
                    img {
                        src: "/media/xrat-logo.png",
                        alt: "XRAT",
                        width: "36",
                        height: "36",
                    }
                    div { class: "sidebar-brand-label flex flex-col min-w-0 gap-[3px] [&_strong]:text-[15px] [&_strong]:tracking-[1.5px] [&_span]:text-[11px] [&_span]:text-muted-foreground",
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
            class: concat!(
                "sidebar-navigation flex flex-1 min-h-0 flex-col overflow-y-auto overflow-x-hidden py-0 px-0.5 gap-1 ",
                "mobile:hidden [&&]:[scrollbar-width:thin] ",
                "[&&]:[scrollbar-color:color-mix(in_srgb,_var(--muted)_35%,_transparent)_transparent] ",
                "[&&_[data-name=SidenavGroupLabel]]:[width:auto] [&&_[data-name=SidenavGroupLabel]]:[max-width:100%] ",
            ),
            aria_label: "Main navigation",
            SidenavContent { class: "gap-4",
                SidenavGroup { class: "p-0",
                    SidenavGroupLabel { class: "sidebar-group-label mt-2.5 mb-1.5 text-muted-foreground text-[11px] font-semibold mx-2.5",
                        "Workspace"
                    }
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
                        SidenavGroupLabel { class: "sidebar-group-label mt-2.5 mb-1.5 text-muted-foreground text-[11px] font-semibold mx-2.5",
                            "Management"
                        }
                        SidenavMenu {
                            NavItem { title: "Host administration", page }
                            NavItem { title: "Settings", page }
                            NavItem { title: "Users", page }
                        }
                    }
                }
                SidenavGroup { class: "p-0",
                    SidenavGroupLabel { class: "sidebar-group-label mt-2.5 mb-1.5 text-muted-foreground text-[11px] font-semibold mx-2.5",
                        "Tools"
                    }
                    SidenavMenu {
                        for title in ["TCP scanner", "Parse configs", "GeoIP lookup"] {
                            NavItem { title, page }
                        }
                    }
                }
            }
        }
        SidenavFooter { class: "sidebar-footer border-t border-solid border-t-border mt-4 pt-3 mobile:hidden",
            div { class: "sidebar-account-row flex items-center gap-1",
                div {
                    class: "sidebar-account-link flex-1 min-w-0 [&_>_.sidebar-account]:w-full",
                    Button {
                        variant: ButtonVariant::Ghost,
                        class: "sidebar-account flex items-center justify-start flex-1 min-w-0 border-0 rounded-[6px] bg-transparent text-foreground text-left h-auto p-1.5 gap-2.5 [&:hover]:bg-accent",
                        aria_label: "Open Account",
                        onclick: move |_| page.set("Account".into()),
                        Avatar {
                            AvatarFallback { UserRound {} }
                        }
                        div { class: "sidebar-account-label flex flex-col min-w-0 gap-[3px] [&_span]:text-[11px] [&_span]:text-muted-foreground [&_strong]:text-[12px] [&_strong]:font-semibold [&_strong]:overflow-hidden [&_strong]:[text-overflow:ellipsis] [&_strong]:whitespace-nowrap",
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
            Tooltip { class: "sidebar-tooltip block w-full m-0",
                SidenavMenuButton {
                    class: concat!(
                        "sidebar-menu-button [&_svg]:size-4.5 [&_svg]:shrink-[0] flex items-center w-full min-h-8.5 border-0 ",
                        "rounded-[6px] bg-transparent text-muted-foreground text-left text-[13px] font-medium py-[7px] px-[9px] ",
                        "gap-[9px] [&:hover]:bg-accent [&:hover]:text-foreground [&&[data-active=true]]:bg-accent ",
                        "[&&[data-active=true]]:text-foreground [&&[data-active=true]]:[font-weight:650] ",
                        "[&&[data-active=true]_svg]:text-primary ",
                    ),
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
