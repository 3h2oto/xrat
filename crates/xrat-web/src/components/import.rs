use crate::api::actions::run;
use crate::ui::button::ButtonVariant;
use crate::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::ui::field::Field;
use crate::ui::icon_button::IconButton;
use crate::ui::input::Input;
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
    let mut dragging = use_signal(|| false);
    let mut file_name = use_signal(String::new);
    let mut file_error = use_signal(String::new);
    let mut reading = use_signal(|| false);
    let mut load_files = move |files: Vec<dioxus::html::FileData>| {
        if files.len() != 1 {
            file_error.set("Choose one text file at a time.".into());
            return;
        }
        let file = files.into_iter().next().unwrap();
        reading.set(true);
        file_error.set(String::new());
        spawn(async move {
            match file.read_string().await {
                Ok(text) => {
                    input.set(text);
                    file_name.set(file.name());
                }
                Err(_) => file_error.set(
                    "Could not read this file. Choose a text file or paste its contents.".into(),
                ),
            }
            reading.set(false);
        });
    };
    crate::hooks::use_import_paste::use_import_paste(input, importing);
    rsx! {
        if importing() {
            Dialog {
                DialogContent {
                    initial_open: true,
                    on_close: move |_| importing.set(false),
                    class: "import-dialog w-[min(36rem,calc(100%-2rem))] flex flex-col gap-5 mobile:p-5",
                    DialogHeader { class: "text-left pr-5",
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
                                "Paste configuration links or a subscription URL, or import a text file."
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
                    div {
                        class: "import-dropzone relative rounded-xl border border-dashed bg-card p-5 text-center transition-colors",
                        "data-dragging": dragging().to_string(),
                        ondragover: move |event: DragEvent| {
                            event.prevent_default();
                            event.data_transfer().set_drop_effect("copy");
                            dragging.set(true);
                        },
                        ondragleave: move |_| dragging.set(false),
                        ondrop: move |event: DragEvent| {
                            event.prevent_default();
                            dragging.set(false);
                            load_files(event.data_transfer().files());
                        },
                        icons::Upload { class: "size-6 text-primary mx-auto mb-2" }
                        label { class: "block font-medium text-sm", r#for: "import-file", "Drop a text file or browse" }
                        p { class: "text-xs text-muted-foreground mt-1", "Your file is read locally before importing." }
                        input {
                            id: "import-file",
                            r#type: "file",
                            class: "absolute inset-0 w-full h-full opacity-0 cursor-pointer focus-visible:outline-offset-2",
                            aria_label: "Choose configuration text file",
                            disabled: reading(),
                            onchange: move |event: FormEvent| load_files(event.files()),
                        }
                    }
                    if reading() {
                        p { role: "status", class: "text-xs text-muted-foreground", "Reading file…" }
                    } else if !file_name().is_empty() {
                        p { role: "status", class: "text-xs text-muted-foreground break-all", "Loaded {file_name()}" }
                    }
                    if !file_error().is_empty() {
                        p { role: "alert", class: "text-sm text-destructive", "{file_error}" }
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
                            class: "min-h-36 max-h-64 font-mono text-xs resize-y",
                            value: input(),
                            oninput: move |e: FormEvent| input.set(e.value()),
                        }
                    }
                    DialogFooter { class: "flex-row justify-end border-t pt-4 gap-2",
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| importing.set(false),
                            icons::X {}
                        }
                        IconButton {
                            label: if section == "Subscriptions" { "Add subscription" } else { "Import configurations" },
                            disabled: reading() || input().trim().is_empty(),
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
