use super::*;

#[component]
pub(crate) fn SignOutButton() -> Element {
    let Session(mut session) = use_context();
    let Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            disabled: pending(),
            onclick: move |_| {
                if *pending.peek() {
                    return;
                }
                pending.set(true);
                spawn(async move {
                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                    match api::post::<_, bool>("logout", &(), &csrf).await {
                        Ok(_) => session.set(None),
                        Err(error) => notice.set(Some(crate::Feedback::error(error))),
                    }
                    pending.set(false);
                });
            },
            if pending() {
                Spinner {}
                "Signing out…"
            } else {
                LogOut {}
                "Sign out"
            }
        }
    }
}
