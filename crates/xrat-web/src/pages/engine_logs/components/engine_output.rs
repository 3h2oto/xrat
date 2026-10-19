use super::*;

#[component]
pub(crate) fn EngineOutput() -> Element {
    let mut tick = use_signal(|| 0_u64);
    let mut manual_reload = use_signal(|| false);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            tick += 1;
        }
    });
    let mut logs = use_resource(move || async move {
        let _ = tick();
        let result = match api::get::<Vec<(String, String)>>("logs/engine").await {
            Err(error) if error == "Request failed (404)." => {
                match api::get::<xrat_web::types::Overview>("overview").await {
                    Ok(overview) if !overview.running => Ok(Vec::new()),
                    _ => Err("Engine logs are unavailable on this server. Restart XRAT with the current build.".into()),
                }
            }
            result => result,
        };
        manual_reload.set(false);
        result
    });
    rsx! {
        Card { class: "log-page-panel min-w-0 max-w-full rounded-[9px] shadow-none border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] p-5 gap-0 mobile:pr-3.5 mobile:pb-3.5 mobile:pt-3.5 mobile:pl-3.5",

            match logs.read().as_ref() {
                None => rsx! {
                    super::super::suspend::Suspend {}
                },
                Some(Err(error)) => rsx! {
                    ErrorState { message: error.clone() }
                    IconButton {
                        label: "Retry",
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            manual_reload.set(true);
                            logs.restart();
                        },
                        RefreshCw {}
                    }
                },
                Some(Ok(files)) => rsx! {
                    LogConsole {
                        id: "engine-console",
                        lines: files.iter().flat_map(|(source, content)| engine_lines(content, source)).collect(),
                        loading: manual_reload(),
                        limit: "Latest session · 64 KiB per stream",
                        empty_hint: "Connect a configuration to start recording engine output.",
                        on_reload: move |_| {
                            manual_reload.set(true);
                            logs.restart();
                        },
                    }
                },
            }
        }
    }
}
