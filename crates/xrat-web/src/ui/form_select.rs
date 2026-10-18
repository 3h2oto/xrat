use super::select::{Select, SelectContent, SelectGroup, SelectOption, SelectTrigger, SelectValue};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct FormSelectOption {
    pub value: String,
    pub label: String,
}

impl FormSelectOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[component]
pub fn FormSelect(
    id: String,
    mut value: Signal<String>,
    options: Vec<FormSelectOption>,
    #[props(default)] disabled: bool,
) -> Element {
    let label = options
        .iter()
        .find(|option| option.value == value())
        .map(|option| option.label.clone());
    rsx! {
        Select {
            class: "w-full",
            selected_value: value(),
            on_change: move |next: Option<String>| {
                if let Some(next) = next {
                    value.set(next);
                }
            },
            SelectTrigger { id: id.clone(), disabled: disabled || options.is_empty(),
                SelectValue { display_value: label, placeholder: "Select an option" }
            }
            SelectContent {
                SelectGroup { aria_label: id,
                    for option in options {
                        SelectOption { key: "{option.value}", value: option.value, "{option.label}" }
                    }
                }
            }
        }
    }
}
