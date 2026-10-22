use super::*;

#[component]
pub(crate) fn HostToggle(
    kind: &'static str,
    mut grant: Signal<Option<HostUnlock>>,
    busy: Signal<bool>,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    let mut tick = use_signal(|| 0);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            tick += 1;
        }
    });
    let mut status = use_resource(move || async move {
        let _ = tick();
        api::get::<xrat_web::types::RuntimeDetails>("overview/runtime").await
    });
    let checked = status
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|runtime| {
            if kind == "rotation" {
                runtime.rotation_enabled
            } else {
                runtime.tun_enabled
            }
        });
    rsx! {
        div { class: "feature-toggle-row flex items-center justify-between mt-auto pt-2 gap-4 [&_>_span]:flex [&_>_span]:items-center [&_>_span]:text-[12px] [&_>_span]:gap-[7px] [&_>_span_svg]:w-3.5 [&_>_span_svg]:h-3.5",
            span { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                if pending() {
                    Spinner {}
                    "Updating…"
                } else if status.read().is_none() {
                    span {
                        role: "status",
                        aria_label: "Loading control status",
                        aria_busy: "true",
                        crate::ui::skeleton::Skeleton { class: "loading-label w-16 h-3" }
                    }
                } else {
                    match checked {
                        Some(true) => "Enabled",
                        Some(false) => "Disabled",
                        None => "State unavailable",
                    }
                }
            }
            crate::ui::switch::Switch {
                checked: checked.unwrap_or(false),
                disabled: checked.is_none() || busy() || pending() || grant().is_none(),
                aria_label: if kind == "rotation" { "Automatic rotation" } else { "TUN capture" },
                on_checked_change: move |enabled| {
                    let Some(unlock) = grant() else {
                        return;
                    };
                    pending.set(true);
                    spawn(async move {
                        let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                        let action = if kind == "rotation" {
                            Action::Rotation { enabled }
                        } else {
                            Action::Tun { enabled }
                        };
                        let result = match api::host_action(&action, &csrf, &unlock.token).await {
                            Ok(job) => api::wait_for_job(&job.id).await,
                            Err(error) => Err(error),
                        };
                        if let Err(error) = result {
                            if error.contains("not permitted") {
                                grant.set(None);
                            }
                            notice.set(Some(crate::Feedback::error(error)));
                        }
                        status.restart();
                        pending.set(false);
                    });
                },
            }
        }
    }
}
