use crate::api::actions::run;
use crate::ui::button::ButtonVariant;
use crate::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::ui::field::Field;
use crate::ui::icon_button::IconButton;
use crate::ui::input::{Input, InputType};
use crate::ui::label::Label;
use crate::ui::textarea::Textarea;
use dioxus::prelude::*;
use xrat_web::types::Action;
#[derive(Clone, Copy)]
pub struct ImportState {
    pub input: Signal<String>,
    pub importing: Signal<bool>,
}

#[component]
pub fn ImportController(section: String) -> Element {
    let state = use_context::<ImportState>();
    let mut input = state.input;
    let mut importing = state.importing;
    let mut name = use_signal(String::new);
    crate::hooks::use_import_paste::use_import_paste(input, importing);
    rsx! {
        span { hidden: true, "data-import-paste": "true" }
        if importing() {
            Dialog {
                DialogContent {
                    initial_open: true,
                    on_close: move |_: MouseEvent| importing.set(false),
                    class: "w-[min(36rem,calc(100%-2rem))]",
                    DialogHeader {
                        DialogTitle {
                            if section == "Subscriptions" {
                                "Add subscription"
                            } else {
                                "Import configurations"
                            }
                        }
                        DialogDescription {
                            if section == "Subscriptions" {
                                "Paste a subscription URL or load its contents from a text file."
                            } else {
                                "Paste a subscription URL or configuration links. Local file paths are not accepted."
                            }
                        }
                    }
                    Field {
                        Label { html_for: "import-name", "Name (optional)" }
                        Input {
                            id: "import-name",
                            value: name(),
                            oninput: move |e: FormEvent| name.set(e.value()),
                        }
                    }
                    Field {
                        Label { html_for: "import-file", "Or load a local text file" }
                        Input {
                            id: "import-file",
                            r#type: InputType::File,
                            onchange: move |event: FormEvent| {
                                if let Some(file) = event.files().into_iter().next() {
                                    spawn(async move {
                                        if let Ok(text) = file.read_string().await {
                                            input.set(text);
                                        }
                                    });
                                }
                            },
                        }
                    }
                    Field {
                        Label { html_for: "import-input",
                            if section == "Subscriptions" {
                                "Subscription URL or content"
                            } else {
                                "URL or configuration content"
                            }
                        }
                        Textarea {
                            id: "import-input",
                            rows: 7,
                            value: input(),
                            oninput: move |e: FormEvent| input.set(e.value()),
                        }
                    }
                    DialogFooter {
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| importing.set(false),
                            icons::X {}
                        }
                        IconButton {
                            label: if section == "Subscriptions" { "Add subscription" } else { "Import configurations" },
                            disabled: input().trim().is_empty(),
                            onclick: move |_| {
                                run(Action::Import {
                                    input: input(),
                                    name: (!name().is_empty()).then_some(name()),
                                });
                                input.set(String::new());
                                importing.set(false);
                            },
                            icons::Plus {}
                        }
                    }
                }
            }
        }
    }
}
