use dioxus::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Skeleton(#[props(into, optional)] class: Option<String>) -> Element {
    let merged_class = tw_merge!(
        concat!(
            "ui-skeleton animate-pulse rounded-md bg-muted ",
            "[&&]:[background-color:color-mix(in_srgb,_var(--muted)_14%,_var(--panel))] ",
            "[&&]:[background-image:linear-gradient(_100deg,_transparent_25%,_color-mix(in_srgb,_var(--foreground)_8%,_transparent)_50%,_transparent_75%_)] ",
            "[&&]:[background-size:200%_100%] [&&]:[animation:skeleton-shimmer_1.8s_ease-in-out_infinite] ",
        ),
        class.as_deref().unwrap_or("")
    );

    rsx! {
        div { "data-name": "Skeleton", class: "{merged_class}" }
    }
}
