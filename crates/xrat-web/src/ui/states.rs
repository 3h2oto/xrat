use super::{
    alert::{Alert, AlertDescription, AlertTitle, AlertVariant},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyTitle},
    skeleton::Skeleton,
    spinner::Spinner,
};
use dioxus::prelude::*;

#[component]
pub fn EmptyState(title: String, description: String) -> Element {
    rsx! {
        Empty { class: "my-4",
            EmptyHeader {
                EmptyTitle { "{title}" }
                EmptyDescription { "{description}" }
            }
        }
    }
}

#[component]
pub fn LoadingState(label: String) -> Element {
    rsx! {
        div {
            class: "space-y-3 py-4",
            role: "status",
            aria_label: label.clone(),
            div { class: "flex items-center gap-2 text-muted-foreground text-sm",
                Spinner {}
                "{label}"
            }
            div { aria_hidden: "true",
                Skeleton { class: "h-8 w-2/3" }
                Skeleton { class: "h-8 w-full mt-3" }
                Skeleton { class: "h-8 w-5/6 mt-3" }
            }
        }
    }
}

#[component]
pub fn ErrorState(message: String) -> Element {
    rsx! {
        Alert { variant: AlertVariant::Destructive,
            AlertTitle { "Something went wrong" }
            AlertDescription { "{message}" }
        }
    }
}
