use crate::ui::icon_button::IconButton;
use crate::ui::{
    avatar::{Avatar, AvatarFallback},
    badge::{Badge, BadgeVariant},
    button::ButtonVariant,
    card::Card,
    dialog::*,
    dropdown_menu::*,
    field::Field,
    form_select::{FormSelect, FormSelectOption},
    input::{Input, InputType},
    label::Label,
    spinner::Spinner,
    states::{EmptyState, ErrorState, LoadingState},
    switch::Switch,
    table::*,
    tooltip::{Tooltip, TooltipContent, TooltipPosition},
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{
    EllipsisVertical, Eye, EyeOff, KeyRound, Lock, Pencil, Plus, RefreshCw, Save, Search, Shield,
    UserRound, X,
};
use xrat_web::types::{Role, SaveUserRequest, User};

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
        div { class: "users-page",
            div { class: "users-toolbar",
                div { class: "users-search",
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
                div { class: "users-role-filter",
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
            Card { class: "users-directory",
                div { class: "users-directory-heading",
                    div {
                        h2 { "Accounts" }
                        if let Some(list) = &data {
                            p { class: "muted",
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
                    LoadingState { label: "Loading accounts…" }
                } else if data.is_some() && filtered.is_empty() {
                    EmptyState {
                        title: "No matching accounts",
                        description: "Try another username or role.",
                    }
                } else if data.is_some() {
                    TableWrapper { class: "users-table-wrapper",
                        Table { class: "users-table",
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
                                            div { class: "users-identity",
                                                Avatar { class: "size-8",
                                                    AvatarFallback { UserRound {} }
                                                }
                                                div {
                                                    div { class: "users-name",
                                                        strong { "{user.username}" }
                                                        if session().is_some_and(|me| me.user.username == user.username) {
                                                            span { "You" }
                                                        }
                                                    }
                                                    if user.configured {
                                                        p { class: "users-account-note",
                                                            Lock {}
                                                            "Managed in host config"
                                                        }
                                                    } else if user.must_change_password {
                                                        p { class: "users-account-note",
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
                                                span { class: "users-managed-access",
                                                    "Enabled"
                                                }
                                            } else {
                                                div { class: "users-access",
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
                                                        class: "users-locked",
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
            p { class: "users-permission-note",
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

#[component]
fn AccountDialog(
    user: Option<User>,
    reset_password: bool,
    busy: bool,
    error: String,
    names: Vec<String>,
    current_username: String,
    on_save: EventHandler<SaveUserRequest>,
    on_close: EventHandler<()>,
) -> Element {
    let mut username = use_signal(|| {
        user.as_ref()
            .map(|user| user.username.clone())
            .unwrap_or_default()
    });
    let mut password = use_signal(String::new);
    let role = use_signal(|| {
        user.as_ref()
            .map(|user| user.role.as_str())
            .unwrap_or("viewer")
            .to_string()
    });
    let mut enabled = use_signal(|| user.as_ref().is_none_or(|user| user.enabled));
    let mut show_password = use_signal(|| false);
    let mut local_error = use_signal(String::new);
    let creating = user.is_none();
    let title = if creating {
        "Create an account"
    } else if reset_password {
        "Reset password"
    } else {
        "Edit account"
    };
    let self_account = username() == current_username;
    rsx! {
        Dialog {
            DialogContent {
                initial_open: true,
                class: "account-dialog w-[min(440px,calc(100vw-2rem))]",
                close_on_backdrop_click: !busy,
                on_close: move |_| on_close.call(()),
                DialogHeader {
                    DialogTitle { "{title}" }
                    DialogDescription {
                        if creating {
                            "Give someone access to your XRAT workspace."
                        } else if reset_password {
                            "Set a temporary password. The user must replace it at the next sign-in."
                        } else {
                            "Change this account's role and access."
                        }
                    }
                }
                form {
                    class: "account-editor",
                    onsubmit: move |event| {
                        event.prevent_default();
                        if busy {
                            return;
                        }
                        if creating && names.contains(&username()) {
                            local_error.set("An account with this username already exists.".into());
                            return;
                        }
                        local_error.set(String::new());
                        on_save
                            .call(SaveUserRequest {
                                username: username(),
                                role: role().parse().unwrap_or(Role::Viewer),
                                enabled: enabled(),
                                password: if creating || reset_password {
                                    Some(password())
                                } else {
                                    None
                                },
                            });
                        password.set(String::new());
                    },
                    Field {
                        Label { html_for: "account-username", "Username" }
                        Input {
                            id: "account-username",
                            required: true,
                            minlength: 3,
                            maxlength: 64,
                            readonly: !creating,
                            disabled: busy,
                            value: username(),
                            autocomplete: "off",
                            oninput: move |event: FormEvent| username.set(event.value()),
                        }
                        if creating {
                            p { class: "account-field-help",
                                "3–64 lowercase letters, numbers, dots, underscores, or hyphens."
                            }
                        }
                    }
                    if !reset_password {
                        Field {
                            Label { html_for: "account-role", "Role" }
                            FormSelect {
                                id: "account-role",
                                value: role,
                                disabled: busy,
                                options: vec![
                                    FormSelectOption::new("viewer", "Viewer"),
                                    FormSelectOption::new("operator", "Operator"),
                                    FormSelectOption::new("admin", "Admin"),
                                ],
                            }
                            p { class: "account-field-help", "{role_help(&role())}" }
                        }
                        if !creating {
                            div { class: "account-access-field",
                                Label { html_for: "account-enabled", "Account enabled" }
                                Switch {
                                    id: "account-enabled",
                                    checked: enabled(),
                                    disabled: busy || self_account,
                                    on_checked_change: move |checked| enabled.set(checked),
                                }
                            }
                        }
                    }
                    if creating || reset_password {
                        Field {
                            Label { html_for: "account-password", "Temporary password" }
                            div { class: "login-password",
                                Input {
                                    id: "account-password",
                                    r#type: if show_password() { InputType::Text } else { InputType::Password },
                                    value: password(),
                                    autocomplete: "new-password",
                                    required: true,
                                    minlength: 12,
                                    maxlength: 1024,
                                    disabled: busy,
                                    oninput: move |event: FormEvent| password.set(event.value()),
                                }
                                IconButton {
                                    label: if show_password() { "Hide password" } else { "Show password" },
                                    variant: ButtonVariant::Ghost,
                                    size: crate::ui::button::ButtonSize::IconXs,
                                    disabled: busy,
                                    aria_pressed: show_password(),
                                    onclick: move |_| show_password.toggle(),
                                    if show_password() {
                                        EyeOff {}
                                    } else {
                                        Eye {}
                                    }
                                }
                            }
                            p { class: "account-field-help", "Use at least 12 characters." }
                        }
                    }
                    if !creating {
                        p { class: "account-session-note",
                            KeyRound {}
                            "Saving signs this account out and revokes its API tokens."
                        }
                    }
                    if !local_error().is_empty() {
                        p { class: "error", role: "alert", "{local_error}" }
                    }
                    if !error.is_empty() {
                        p { class: "error", role: "alert", "{error}" }
                    }
                    div { class: "account-dialog-actions",
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Ghost,
                            disabled: busy,
                            onclick: move |_| on_close.call(()),
                            icons::X {}
                        }
                        IconButton {
                            label: if busy { "Saving account" } else if creating { "Create account" } else if reset_password { "Reset password" } else { "Save changes" },
                            button_type: "submit",
                            disabled: busy,
                            if busy {
                                Spinner {}
                            } else if creating {
                                Plus {}
                            } else if reset_password {
                                KeyRound {}
                            } else {
                                Save {}
                            }
                        }
                    }
                }
            }
        }
    }
}
fn role_label(role: Role) -> &'static str {
    match role {
        Role::Viewer => "Viewer",
        Role::Operator => "Operator",
        Role::Admin => "Admin",
    }
}
fn role_help(role: &str) -> &'static str {
    match role {
        "operator" => "Manage configurations, connections, tests, and tools.",
        "admin" => "All operator permissions, plus accounts, settings, and host controls.",
        _ => "Inspect configurations and runtime activity without making changes.",
    }
}
