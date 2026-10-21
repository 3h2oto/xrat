use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum LoadingLayout {
    Dashboard,
    #[default]
    Summary,
}

#[component]
pub fn LoadingState(label: String, #[props(default)] layout: LoadingLayout) -> Element {
    rsx! {
        div {
            class: "loading-state w-full min-w-0",
            role: "status",
            aria_busy: "true",
            aria_label: label.clone(),
            span { class: "sr-only", "{label}" }
            div {
                class: "loading-shapes w-full min-w-0 grid py-1 gap-4",
                aria_hidden: "true",
                match layout {
                    LoadingLayout::Dashboard => rsx! {
                        div { class: "loading-summary-grid grid grid-cols-[repeat(3,_minmax(0,_1fr))] gap-4 mobile:grid-cols-[minmax(0,_1fr)] mobile:[&_.loading-card:not(:first-child)]:hidden",
                            for index in 0..3 {
                                div {
                                    key: "{index}",
                                    class: "loading-card grid border border-solid border-border rounded-[10px] p-4 gap-4",
                                    Skeleton { class: "loading-heading w-[42%] h-3.5" }
                                    Skeleton { class: "loading-summary h-29 mobile:h-18" }
                                    Skeleton { class: "loading-line w-full h-3" }
                                }
                            }
                        }
                        div { class: "loading-chart-grid grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-4 mobile:grid-cols-[minmax(0,_1fr)]",
                            for index in 0..2 {
                                div {
                                    key: "{index}",
                                    class: "loading-card grid border border-solid border-border rounded-[10px] p-4 gap-4",
                                    Skeleton { class: "loading-heading w-[42%] h-3.5" }
                                    Skeleton { class: "loading-chart h-[clamp(150px,_22dvh,_260px)]" }
                                }
                            }
                        }
                    },
                    LoadingLayout::Summary => rsx! {
                        div { class: "loading-fields grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-4",
                            for index in 0..4 {
                                div { key: "{index}", class: "loading-field grid gap-2.5",
                                    Skeleton { class: "loading-label w-16 h-3" }
                                    Skeleton { class: "loading-line w-full h-3" }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
pub fn SessionSkeleton() -> Element {
    rsx! {
        div { class: "session-skeleton grid grid-cols-[68px_minmax(0,_1fr)] w-full min-h-dvh mobile:grid-cols-[minmax(0,_1fr)]",
            aside {
                class: "session-skeleton-sidebar flex flex-col items-center py-5 px-3 gap-5 mobile:hidden [&_img]:mb-3",
                aria_hidden: "true",
                img {
                    src: "/media/xrat-logo.png",
                    alt: "",
                    width: "32",
                    height: "32",
                }
                for index in 0..7 {
                    Skeleton { key: "{index}", class: "loading-nav-icon size-7" }
                }
            }
            main { class: "session-skeleton-main w-full max-w-450 py-6 px-[clamp(16px,_4vw,_64px)]",
                div {
                    class: "session-skeleton-heading grid mb-6 gap-3",
                    aria_hidden: "true",
                    Skeleton { class: "loading-page-title w-45 max-w-[60%] h-6.5" }
                    Skeleton { class: "loading-page-caption w-75 max-w-[85%] h-3" }
                }
                LoadingState {
                    label: "Loading your workspace",
                    layout: LoadingLayout::Dashboard,
                }
            }
        }
    }
}

#[component]
pub fn LoadingRegion(label: String, #[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "loading-state min-w-0 w-full {class}", role: "status", aria_busy: "true", aria_label: label.clone(),
            span { class: "sr-only", "{label}" }
            div { aria_hidden: "true", {children} }
        }
    }
}

#[component]
pub fn TableSkeleton(columns: String, cells: usize, #[props(default = 6)] rows: usize) -> Element {
    rsx! {
        div { class: "overflow-x-auto",
            div { class: "min-w-160",
                div { class: "grid {columns} gap-4 border-b border-border py-3 px-2",
                    for column in 0..cells {
                        Skeleton { key: "heading-{column}", class: "h-3 w-3/4" }
                    }
                }
                for row in 0..rows {
                    div { key: "{row}", class: "grid {columns} items-center gap-4 border-b border-border/40 py-4 px-2",
                        for column in 0..cells {
                            Skeleton { key: "{column}", class: if (row + column) % 3 == 0 { "h-4 w-2/3" } else { "h-4 w-full" } }
                        }
                    }
                }
            }
        }
    }
}
