use super::components::*;
use crate::components::states::{EmptyState, ErrorState};
use crate::ui::icon_button::IconButton;
use crate::ui::{
    avatar::{Avatar, AvatarFallback},
    badge::{Badge, BadgeVariant},
    button::ButtonVariant,
    card::Card,
    dropdown_menu::*,
    form_select::{FormSelect, FormSelectOption},
    input::Input,
    label::Label,
    spinner::Spinner,
    switch::Switch,
    table::*,
    tooltip::{Tooltip, TooltipContent, TooltipPosition},
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{
    EllipsisVertical, KeyRound, Lock, Pencil, Plus, RefreshCw, Search, Shield, UserRound, X,
};
use xrat_web::types::{SaveUserRequest, User};

#[component]
pub fn Users() -> Element {
    let Session(mut session) = use_context();
    let Notice(mut notice) = use_context();
    let mut users = use_signal(|| None::<Vec<User>>);
    let mut loading = use_signal(|| true);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut form_error = use_signal(String::new);
    let mut query = use_signal(String::new);
    let role_filter = use_signal(|| "all".to_string());
    let mut creating = use_signal(|| false);
    let mut editing = use_signal(|| None::<User>);
    let mut resetting = use_signal(|| false);
    use_future(move || async move {
        match api::get::<Vec<User>>("users").await {
            Ok(data) => users.set(Some(data)),
            Err(message) => error.set(message),
        }
        loading.set(false);
    });
    let save = use_callback(move |request: SaveUserRequest| {
        if busy() {
            return;
        }
        busy.set(true);
        form_error.set(String::new());
        spawn(async move {
            let me = session();
            let csrf = me
                .as_ref()
                .map(|me| me.csrf_token.clone())
                .unwrap_or_default();
            match api::post::<_, bool>("users", &request, &csrf).await {
                Ok(_) => {
                    creating.set(false);
                    editing.set(None);
                    if me.is_some_and(|me| me.user.username == request.username) {
                        notice.set(Some(crate::Feedback::info(
                            "Account updated. Sign in again to continue.",
                        )));
                        session.set(None);
                    } else {
                        notice.set(Some(crate::Feedback::success("Account saved.")));
                        match api::get::<Vec<User>>("users").await {
                            Ok(data) => users.set(Some(data)),
                            Err(message) => error.set(message),
                        }
                    }
                }
                Err(message) => {
                    form_error.set(message.clone());
                    notice.set(Some(crate::Feedback::error(message)));
                }
            }
            busy.set(false);
        });
    });
    let data = users();
    let needle = query().to_lowercase();
    let filtered = data
        .as_ref()
        .map(|list| {
            list.iter()
                .filter(|user| {
                    user.username.to_lowercase().contains(&needle)
                        && (role_filter() == "all" || user.role.as_str() == role_filter())
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    rsx! {
        div { class: "users-page max-w-320 mx-auto",
            div { class: "users-toolbar flex items-center mb-5 gap-2.5 [@media(width<=600px)]:flex-wrap [&_>_button]:ml-auto [&_>_.icon-action]:ml-auto",
                div {
                    class: concat!(
                        "users-search relative flex items-center flex-1 max-w-100 min-w-0 ",
                        "[@media(width<=600px)]:max-w-none [@media(width<=600px)]:basis-full [&_>_svg]:absolute ",
                        "[&_>_svg]:left-2.5 [&_>_svg]:w-[15px] [&_>_svg]:h-[15px] [&_>_svg]:text-muted-foreground ",
                        "[&_>_svg]:pointer-events-none [&_input]:h-8 [&_input]:text-[12px] [&_input]:pl-8 ",
                        "[&_input]:pr-7.5 [&_>_button]:absolute [&_>_button]:right-[5px] [&_>_button]:grid ",
                        "[&_>_button]:place-items-center [&_>_button]:w-[23px] [&_>_button]:h-[23px] ",
                        "[&_>_button]:border-0 [&_>_button]:rounded-[4px] [&_>_button]:bg-transparent ",
                        "[&_>_button]:text-muted-foreground [&_>_button:hover]:bg-accent [&_>_button_svg]:w-3 ",
                        "[&_>_button_svg]:h-3 [&_>_.icon-action]:absolute [&_>_.icon-action]:right-[5px] ",
                        "[&_>_.icon-action]:top-[50%] [&_>_.icon-action]:transform-[translateY(-50%)]",
                    ),
                    Search {}
                    Input {
                        id: "users-search",
                        aria_label: "Search accounts",
                        placeholder: "Search accounts…",
                        disabled: data.is_none(),
                        value: query(),
                        oninput: move |event: FormEvent| query.set(event.value()),
                    }
                    if !query().is_empty() {
                        IconButton {
                            label: "Clear search",
                            variant: ButtonVariant::Ghost,
                            size: crate::ui::button::ButtonSize::IconXs,
                            onclick: move |_| query.set(String::new()),
                            X {}
                        }
                    }
                }
                div { class: "users-role-filter w-32.5 shrink-0 [&&_[data-name=SelectTrigger]]:[height:32px] [&&_[data-name=SelectTrigger]]:[font-size:12px]",
                    Label { class: "sr-only", html_for: "users-role-filter", "Filter by role" }
                    FormSelect {
                        id: "users-role-filter",
                        value: role_filter,
                        disabled: data.is_none(),
                        options: vec![
                            FormSelectOption::new("all", "All roles"),
                            FormSelectOption::new("viewer", "Viewer"),
                            FormSelectOption::new("operator", "Operator"),
                            FormSelectOption::new("admin", "Admin"),
                        ],
                    }
                }
                IconButton {
                    label: "New user",
                    disabled: busy() || data.is_none(),
                    onclick: move |_| {
                        editing.set(None);
                        resetting.set(false);
                        form_error.set(String::new());
                        creating.set(true);
                    },
                    Plus {}
                }
            }
            if !error().is_empty() {
                ErrorState { message: error() }
            }
            Card {
                class: concat!(
                    "users-directory shadow-none rounded-[9px] ",
                    "border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] p-0 gap-0 ",
                    "[&_.users-table-wrapper]:border-0 [&_.users-table-wrapper]:border-t ",
                    "[&_.users-table-wrapper]:border-solid ",
                    "[&_.users-table-wrapper]:border-t-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] ",
                    "[&_.users-table-wrapper]:rounded-[0_0_9px_9px] [&_.users-table-wrapper]:max-h-[60dvh]",
                ),
                div { class: "users-directory-heading flex items-center justify-between p-4.5 gap-4 [&_h2]:text-[16px] [&_h2]:m-0 [&_p]:mt-[3px] [&_p]:mb-0 [&_p]:text-[11px] [&_p]:tabular-nums [&_p]:mx-0",
                    div {
                        h2 { "Accounts" }
                        if let Some(list) = &data {
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                "{list.len()} total · {list.iter().filter(|user| user.enabled).count()} enabled"
                            }
                        }
                    }
                    IconButton {
                        label: "Reload accounts",
                        variant: ButtonVariant::Ghost,
                        disabled: loading() || busy(),
                        onclick: move |_| {
                            loading.set(true);
                            error.set(String::new());
                            spawn(async move {
                                match api::get::<Vec<User>>("users").await {
                                    Ok(data) => users.set(Some(data)),
                                    Err(message) => error.set(message),
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
                }
                if data.is_none() && loading() {
                    super::suspend::Suspend {}
                } else if data.is_some() && filtered.is_empty() {
                    EmptyState {
                        title: "No matching accounts",
                        description: "Try another username or role.",
                    }
                } else if data.is_some() {
                    TableWrapper { class: "users-table-wrapper",
                        Table {
                            class: concat!(
                                "users-table [&_th]:align-middle [&_th]:text-left [&_th]:text-[11px] [&_th]:font-medium ",
                                "[&_th]:py-3 [&_th]:px-4.5 [@media(width<=600px)]:[&_th]:pr-3 ",
                                "[@media(width<=600px)]:[&_th]:pb-2.5 [@media(width<=600px)]:[&_th]:pt-2.5 ",
                                "[@media(width<=600px)]:[&_th]:pl-3 [&_td]:align-middle [&_td]:text-left [&_td]:text-[12px] ",
                                "[&_td]:py-3 [&_td]:px-4.5 [@media(width<=600px)]:[&_td]:pr-3 ",
                                "[@media(width<=600px)]:[&_td]:pb-2.5 [@media(width<=600px)]:[&_td]:pt-2.5 ",
                                "[@media(width<=600px)]:[&_td]:pl-3 [&_.users-actions-cell]:text-right ",
                                "[&_.users-actions-cell]:w-13.5 [&_.users-actions-heading]:text-right ",
                                "[&_.users-actions-heading]:w-13.5",
                            ),
                            TableHeader {
                                TableRow {
                                    TableHead { "Account" }
                                    TableHead { "Role" }
                                    TableHead { "Access" }
                                    TableHead { class: "users-actions-heading",
                                        span { class: "sr-only", "Actions" }
                                    }
                                }
                            }
                            TableBody {
                                for user in filtered {
                                    TableRow { key: "{user.username}",
                                        TableCell {
                                            div { class: "users-identity flex items-center min-w-35 gap-2.5 [&&_[data-slot=avatar-fallback]_svg]:[width:15px] [&&_[data-slot=avatar-fallback]_svg]:[height:15px]",
                                                Avatar { class: "size-8",
                                                    AvatarFallback { UserRound {} }
                                                }
                                                div {
                                                    div { class: "users-name flex items-center gap-2 [&_strong]:font-[550] [&_strong]:text-[13px] [&_>_span]:text-[10px] [&_>_span]:text-muted-foreground",
                                                        strong { "{user.username}" }
                                                        if session().is_some_and(|me| me.user.username == user.username) {
                                                            span { "You" }
                                                        }
                                                    }
                                                    if user.configured {
                                                        p { class: "users-account-note flex items-center text-muted-foreground text-[10px] mt-1 mb-0 whitespace-nowrap mx-0 gap-1 [&_svg]:w-2.5 [&_svg]:h-2.5",
                                                            Lock {}
                                                            "Managed in host config"
                                                        }
                                                    } else if user.must_change_password {
                                                        p { class: "users-account-note flex items-center text-muted-foreground text-[10px] mt-1 mb-0 whitespace-nowrap mx-0 gap-1 [&_svg]:w-2.5 [&_svg]:h-2.5",
                                                            KeyRound {}
                                                            "Password change required"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        TableCell {
                                            Badge { variant: if user.role.is_admin() { BadgeVariant::Secondary } else { BadgeVariant::Outline },
                                                "{role_label(user.role)}"
                                            }
                                        }
                                        TableCell {
                                            if user.configured {
                                                span { class: "users-managed-access text-[11px] text-muted-foreground",
                                                    "Enabled"
                                                }
                                            } else {
                                                div { class: "users-access flex items-center gap-2 [&_>_span]:text-[11px] [&_>_span]:text-muted-foreground",
                                                    Switch {
                                                        checked: user.enabled,
                                                        disabled: busy() || session().is_some_and(|me| me.user.username == user.username),
                                                        aria_label: format!("Enable account {}", user.username),
                                                        on_checked_change: {
                                                            let user = user.clone();
                                                            move |enabled| {
                                                                save
                                                                    .call(SaveUserRequest {
                                                                    username: user.username.clone(),
                                                                    role: user.role,
                                                                    enabled,
                                                                    password: None,
                                                                })
                                                            }
                                                        },
                                                    }
                                                    span {
                                                        if user.enabled {
                                                            "Enabled"
                                                        } else {
                                                            "Disabled"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        TableCell { class: "users-actions-cell",
                                            if user.configured {
                                                Tooltip { class: "m-0",
                                                    span {
                                                        class: "users-locked inline-flex items-center justify-center size-7 text-muted-foreground rounded-[4px] [&_svg]:size-[13px]",
                                                        tabindex: "0",
                                                        aria_label: "Managed in host configuration",
                                                        Lock {}
                                                    }
                                                    TooltipContent { position: TooltipPosition::Top,
                                                        "This account is managed in host configuration."
                                                    }
                                                }
                                            } else {
                                                DropdownMenu {
                                                    DropdownMenuTrigger { as_child: true,
                                                        IconButton {
                                                            label: format!("Actions for account {}", user.username),
                                                            variant: ButtonVariant::Ghost,
                                                            EllipsisVertical {}
                                                        }
                                                    }
                                                    DropdownMenuContent {
                                                        DropdownMenuLabel { "{user.username}" }
                                                        DropdownMenuGroup {
                                                            DropdownMenuAction {
                                                                disabled: busy(),
                                                                onclick: {
                                                                    let user = user.clone();
                                                                    move |_| {
                                                                        editing.set(Some(user.clone()));
                                                                        creating.set(false);
                                                                        resetting.set(false);
                                                                        form_error.set(String::new());
                                                                    }
                                                                },
                                                                Pencil {}
                                                                "Edit account"
                                                            }
                                                            DropdownMenuAction {
                                                                disabled: busy(),
                                                                onclick: {
                                                                    let user = user.clone();
                                                                    move |_| {
                                                                        editing.set(Some(user.clone()));
                                                                        creating.set(false);
                                                                        resetting.set(true);
                                                                        form_error.set(String::new());
                                                                    }
                                                                },
                                                                KeyRound {}
                                                                "Reset password"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "users-permission-note flex items-start text-[11px] text-muted-foreground leading-[1.7] mt-4.5 mb-0 mx-0 gap-2 [&_svg]:size-3.5 [&_svg]:shrink-[0] [&_svg]:mt-0.5",
                Shield {}
                "Viewer: inspect · Operator: manage connections · Admin: manage accounts, settings, and host controls"
            }
            if creating() || editing().is_some() {
                AccountDialog {
                    key: "{creating}:{resetting}:{editing().map(|user| user.username).unwrap_or_default()}",
                    user: editing(),
                    reset_password: resetting(),
                    busy: busy(),
                    error: form_error(),
                    names: data.clone()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|user| user.username)
                        .collect::<Vec<_>>(),
                    current_username: session().map(|me| me.user.username).unwrap_or_default(),
                    on_save: save,
                    on_close: move |_| {
                        creating.set(false);
                        editing.set(None);
                        form_error.set(String::new());
                    },
                }
            }
        }
    }
}
