use crate::ui::{
    alert::{Alert, AlertDescription, AlertTitle, AlertVariant},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyTitle},
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

pub use super::loading::{LoadingLayout, LoadingState};

#[component]
pub fn ErrorState(message: String) -> Element {
    rsx! {
        Alert { variant: AlertVariant::Destructive,
            AlertTitle { "Something went wrong" }
            AlertDescription { "{message}" }
        }
    }
}
