use super::components::*;
use crate::Session;
use dioxus::prelude::*;

#[component]
pub fn Account() -> Element {
    let Session(session) = use_context();
    let Some(me) = session() else { return rsx! {} };
    let show_tokens = me.user.role.can_operate() && !me.user.must_change_password;
    rsx! {
        div { class: "self-account-page max-w-320 mx-auto",
            AccountSummary { user: me.user.clone() }
            PasswordSettings {
                configured: me.user.configured,
                required: me.user.must_change_password,
                show_signout: !show_tokens,
            }
            if show_tokens {
                ApiTokens {}
            }
        }
    }
}
