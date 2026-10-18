use crate::ui::card::Card;
use crate::ui::history::{TimeFilter, timestamp, use_time_window};
use crate::ui::spinner::Spinner;
use crate::ui::states::{EmptyState, ErrorState, LoadingLayout, LoadingState};
use crate::ui::table::*;
use crate::ui::tooltip::{Tooltip, TooltipContent, TooltipPosition};
use crate::{
    Notice, Session, api,
    ui::{button::ButtonVariant, icon_button::IconButton},
};
use dioxus::prelude::*;
use icons::{Ban, CircleCheck, CircleX, Clock};
use xrat_web::types::Job;
#[component]
pub fn Jobs() -> Element {
    let window = use_time_window("24");
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let pushed = use_signal(|| None::<Vec<Job>>);
    let mut live = use_signal(|| false);
    let mut seen = use_signal(|| false);
    let _stream = use_hook(move || std::rc::Rc::new(JobStream::new(pushed, live, seen)));
    let mut tick = use_signal(|| 0);
    use_future(move || async move {
        loop {
            seen.set(false);
            gloo_timers::future::TimeoutFuture::new(5000).await;
            if !seen() {
                live.set(false);
            }
            tick += 1;
        }
    });
    let data = use_resource(move || async move {
        let _ = tick();
        if live() {
            return pushed().ok_or_else(|| "Waiting for job stream".into());
        }
        api::get::<Vec<Job>>("jobs").await
    });
    let current_user = session().map(|session| session.user.username);
    rsx! {
        Card { class: "panel mb-5 p-5 mobile:pr-4 mobile:pb-4 mobile:pt-4 mobile:pl-4",
            div { class: "section-heading flex flex-wrap items-center justify-between mb-4 gap-3 [&_h2]:text-[17px] [&_h2]:m-0",
                h2 { "Background jobs" }
                span { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                    "Live updates with polling fallback"
                }
            }
            match data.read().as_ref() {
                Some(Ok(jobs)) => rsx! {
                    if jobs.is_empty() {
                        EmptyState {
                            title: "No jobs yet",
                            description: "Imports, tests and scans will appear here.",
                        }
                    } else {
                        TimeFilter { window, id: "jobs-time" }
                        if jobs.iter().any(|job| window.contains(job.created_at as f64)) {
                            TableWrapper {
                                class: "jobs-table-wrapper max-h-[min(55dvh,_560px)] min-h-0 mt-3.5 overflow-auto",
                                aria_label: "Background jobs",
                                Table {
                                    class: concat!(
                                        "jobs-table [&_th]:text-left [&_th]:align-middle [&_th]:text-[12px] [&_th]:whitespace-nowrap ",
                                        "[&_th]:py-2 [&_th]:px-2.5 [&_td]:text-left [&_td]:align-middle [&_td]:text-[12px] ",
                                        "[&_td]:leading-[20px] [&_td]:wrap-anywhere [&_td]:py-2 [&_td]:px-2.5 [&_th:first-child]:w-15.5 ",
                                        "[&_th:first-child]:text-center [&_td:first-child]:w-15.5 [&_td:first-child]:text-center ",
                                        "[&_.job-result]:leading-[20px] [&_.job-result]:max-h-22 [&_.job-result]:overflow-auto ",
                                        "[&_.job-result]:whitespace-pre-wrap [&_.job-result]:wrap-anywhere table-fixed min-w-185 ",
                                        "max-w-none w-full [&_th:nth-child(2)]:w-37.5 [&_th:nth-child(3)]:w-[175px] ",
                                        "[&_th:nth-child(4)]:w-22.5 [&_th:last-child]:w-16 [&_th:last-child]:text-center ",
                                        "[&_td:last-child]:text-center",
                                    ),
                                    TableHeader {
                                        TableRow {
                                            TableHead { "Status" }
                                            TableHead { "Job" }
                                            TableHead { "Started (local)" }
                                            TableHead { "User" }
                                            TableHead { "Result" }
                                            TableHead { "Actions" }
                                        }
                                    }
                                    TableBody {
                                        for job in jobs.iter().filter(|job| window.contains(job.created_at as f64)) {
                                            TableRow { key: "{job.id}",
                                                TableCell {
                                                    JobStatus { state: job.state.clone() }
                                                }
                                                TableCell { {crate::ui::job_labels::job_label(&job.kind)} }
                                                TableCell {
                                                    time {
                                                        class: "job-time whitespace-nowrap text-muted-foreground [font-family:inherit] text-[12px] tabular-nums",
                                                        datetime: js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(job.created_at as f64 * 1000.0))
                                                            .to_iso_string()
                                                            .as_string(),
                                                        {timestamp(job.created_at)}
                                                    }
                                                }
                                                TableCell {
                                                    if current_user.as_deref() == Some(job.username.as_str()) {
                                                        "You"
                                                    } else {
                                                        "{job.username}"
                                                    }
                                                }
                                                TableCell {
                                                    div { class: "job-result max-w-112 max-h-32 overflow-auto whitespace-pre-wrap wrap-anywhere",
                                                        {job.result.as_deref().unwrap_or("—")}
                                                    }
                                                }
                                                TableCell {
                                                    if job.state == "running" && matches!(job.kind.as_str(), "test" | "scan" | "bulk") {
                                                        IconButton {
                                                            label: "Cancel job",
                                                            variant: ButtonVariant::Outline,
                                                            onclick: {
                                                                let id = job.id.clone();
                                                                move |_| {
                                                                    let id = id.clone();
                                                                    spawn(async move {
                                                                        let csrf = session().map(|s| s.csrf_token).unwrap_or_default();
                                                                        match api::post::<_, bool>(&format!("jobs/{id}/cancel"), &(), &csrf)
                                                                            .await
                                                                        {
                                                                            Ok(_) => {
                                                                                notice
                                                                                    .set(
                                                                                        Some(crate::Feedback::success("Cancellation requested.")),
                                                                                    )
                                                                            }
                                                                            Err(error) => notice.set(Some(crate::Feedback::error(error))),
                                                                        }
                                                                    });
                                                                }
                                                            },
                                                            Ban {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !jobs.iter().any(|job| window.contains(job.created_at as f64)) {
                            EmptyState {
                                title: "No jobs in this time range",
                                description: "Choose a wider range to see earlier jobs.",
                            }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    ErrorState { message: error.clone() }
                },
                None => rsx! {
                    LoadingState { label: "Loading jobs…", layout: LoadingLayout::Table }
                },
            }
        }
    }
}

struct JobStream {
    source: Option<web_sys::EventSource>,
    _message: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
}
impl JobStream {
    fn new(
        mut pushed: Signal<Option<Vec<Job>>>,
        mut live: Signal<bool>,
        mut seen: Signal<bool>,
    ) -> Self {
        use wasm_bindgen::{JsCast, closure::Closure};
        let message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if let Some(text) = event.data().as_string()
                && let Ok(jobs) = serde_json::from_str(&text)
            {
                pushed.set(Some(jobs));
                seen.set(true);
                live.set(true);
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        let error =
            Closure::wrap(Box::new(move |_: web_sys::Event| live.set(false))
                as Box<dyn FnMut(web_sys::Event)>);
        let source = web_sys::EventSource::new("/api/v1/jobs/events").ok();
        if let Some(source) = &source {
            source.set_onmessage(Some(message.as_ref().unchecked_ref()));
            source.set_onerror(Some(error.as_ref().unchecked_ref()));
        }
        Self {
            source,
            _message: message,
            _error: error,
        }
    }
}
impl Drop for JobStream {
    fn drop(&mut self) {
        if let Some(source) = &self.source {
            source.set_onmessage(None);
            source.set_onerror(None);
            source.close();
        }
    }
}

#[component]
fn JobStatus(state: String) -> Element {
    let tip = use_hook(|| crate::hooks::use_random::use_random_id_for("job-status"));
    rsx! {
        Tooltip { class: "job-status-tooltip inline-flex align-top m-0",
            span {
                class: concat!(
                    "job-status inline-flex size-6 justify-center items-center text-muted-foreground rounded-[4px] ",
                    "[&_svg]:size-4 [&&[data-status=succeeded]]:text-primary [&&[data-status=failed]]:text-destructive ",
                    "[&&[data-status=interrupted]]:text-destructive [&&[data-status=running]]:text-primary ",
                ),
                "data-status": state.clone(),
                role: "img",
                aria_label: state.clone(),
                aria_describedby: tip.clone(),
                tabindex: "0",
                match state.as_str() {
                    "running" => rsx! {
                        Spinner {}
                    },
                    "succeeded" => rsx! {
                        CircleCheck {}
                    },
                    "failed" | "interrupted" => rsx! {
                        CircleX {}
                    },
                    "cancelled" => rsx! {
                        Ban {}
                    },
                    _ => rsx! {
                        Clock {}
                    },
                }
            }
            TooltipContent { id: tip.clone(), position: TooltipPosition::Top, "{state}" }
        }
    }
}
