use super::{
    form_select::{FormSelect, FormSelectOption},
    input::{Input, InputType},
    label::Label,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct TimeWindow {
    pub preset: Signal<String>,
    pub from: Signal<String>,
    pub until: Signal<String>,
}
impl TimeWindow {
    pub fn valid(self) -> bool {
        if (self.preset)() != "custom" {
            return true;
        }
        let from = (self.from)();
        let until = (self.until)();
        let start = parse_time(&from);
        let end = parse_time(&until);
        (from.is_empty() || start.is_some())
            && (until.is_empty() || end.is_some())
            && start.zip(end).is_none_or(|(a, b)| a <= b)
    }
    pub fn contains(self, seconds: f64) -> bool {
        self.contains_at(seconds, js_sys::Date::now() / 1000.0)
    }
    pub fn contains_at(self, seconds: f64, now: f64) -> bool {
        if !self.valid() {
            return false;
        }
        let (from, until) = if (self.preset)() == "custom" {
            (
                parse_time(&(self.from)()).unwrap_or(f64::NEG_INFINITY),
                parse_time(&(self.until)()).unwrap_or(now),
            )
        } else {
            (
                now - (self.preset)().parse::<f64>().unwrap_or(24.0) * 3600.0,
                now,
            )
        };
        seconds >= from && seconds <= until
    }
}
pub fn use_time_window(default_hours: &'static str) -> TimeWindow {
    TimeWindow {
        preset: use_signal(|| default_hours.into()),
        from: use_signal(String::new),
        until: use_signal(String::new),
    }
}
pub fn parse_time(value: &str) -> Option<f64> {
    let value = js_sys::Date::parse(value) / 1000.0;
    value.is_finite().then_some(value)
}
pub fn timestamp(seconds: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(seconds as f64 * 1000.0));
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{} {} {} · {:02}:{:02}:{:02}",
        date.get_date(),
        months[date.get_month() as usize],
        date.get_full_year(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

pub fn local_log_timestamp(seconds: f64) -> Option<String> {
    if !seconds.is_finite() {
        return None;
    }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(seconds * 1000.0));
    if !date.get_time().is_finite() {
        return None;
    }
    Some(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    ))
}
#[component]
pub fn TimeFilter(mut window: TimeWindow, id: String) -> Element {
    rsx! {
        div { class: concat!(
            "history-time-filter flex items-end flex-wrap gap-3 [&_>_div]:min-w-40 [&_label]:block ",
            "[&_label]:text-[11px] [&_label]:font-medium [&_label]:text-muted-foreground [&_label]:mb-[5px] ",
            "[&&_[data-name=SelectTrigger]]:[height:30px] [&&_[data-name=SelectTrigger]]:[font-size:12px] ",
            "[&&_[data-name=SelectTrigger]]:[padding:0_9px] [&&_[data-name=SelectTrigger]]:[gap:8px] ",
        ),
            div {
                Label { html_for: id.clone(), "Time range" }
                FormSelect {
                    id: id.clone(),
                    value: window.preset,
                    options: vec![
                        FormSelectOption::new("0.25", "Last 15 minutes"),
                        FormSelectOption::new("1", "Last hour"),
                        FormSelectOption::new("24", "Last 24 hours"),
                        FormSelectOption::new("168", "Last 7 days"),
                        FormSelectOption::new("custom", "Custom range"),
                    ],
                }
            }
            if !window.valid() {
                p {
                    role: "alert",
                    class: "error text-destructive text-[14px]",
                    "Choose valid dates with From before Until."
                }
            }
            if (window.preset)() == "custom" {
                div {
                    Label { html_for: format!("{id}-from"), "From (local time)" }
                    Input {
                        id: format!("{id}-from"),
                        r#type: InputType::DatetimeLocal,
                        value: (window.from)(),
                        oninput: move |e: FormEvent| window.from.set(e.value()),
                    }
                }
                div {
                    Label { html_for: format!("{id}-until"), "Until (local time)" }
                    Input {
                        id: format!("{id}-until"),
                        r#type: InputType::DatetimeLocal,
                        value: (window.until)(),
                        oninput: move |e: FormEvent| window.until.set(e.value()),
                    }
                }
            }
        }
    }
}

pub fn event_time(value: &str) -> Option<f64> {
    parse_time(&normalize_event_timestamp(value))
}

fn normalize_event_timestamp(value: &str) -> String {
    let mut normalized = value.replacen(' ', "T", 1);
    if normalized
        .get(19..)
        .is_some_and(|tail| !tail.contains(['Z', 'z', '+', '-']))
    {
        normalized.push('Z');
    } else if normalized
        .as_bytes()
        .get(normalized.len().saturating_sub(3))
        .is_some_and(|byte| *byte == b'+' || *byte == b'-')
    {
        normalized.push_str(":00");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::normalize_event_timestamp;

    #[test]
    fn database_timestamps_keep_their_utc_or_explicit_offset_semantics() {
        for (input, expected) in [
            ("2026-10-18 10:20:30", "2026-10-18T10:20:30Z"),
            ("2026-10-18 10:20:30.123456", "2026-10-18T10:20:30.123456Z"),
            (
                "2026-10-18 10:20:30.123+00",
                "2026-10-18T10:20:30.123+00:00",
            ),
            ("2026-10-18T10:20:30+03:30", "2026-10-18T10:20:30+03:30"),
            ("2026-10-18T10:20:30Z", "2026-10-18T10:20:30Z"),
        ] {
            assert_eq!(normalize_event_timestamp(input), expected);
        }
    }
}
