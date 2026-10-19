use super::*;

#[component]
pub(crate) fn ConfigToggle(id: i64, enabled: bool) -> Element {
    let Session(session) = use_context();
    let crate::Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    rsx! {
        crate::ui::switch::Switch {
            checked: enabled,
            disabled: pending(),
            aria_label: format!("Enable configuration {id}"),
            on_checked_change: move |checked| {
                pending.set(true);
                spawn(async move {
                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                    let action = if checked {
                        Action::Enable { id }
                    } else {
                        Action::Disable { id }
                    };
                    let result = match api::post::<
                        _,
                        xrat_web::types::Job,
                    >("actions", &action, &csrf)
                        .await
                    {
                        Ok(job) => api::wait_for_job(&job.id).await,
                        Err(error) => Err(error),
                    };
                    if let Err(error) = result {
                        notice.set(Some(crate::Feedback::error(error)));
                    }
                    pending.set(false);
                });
            },
        }
    }
}
