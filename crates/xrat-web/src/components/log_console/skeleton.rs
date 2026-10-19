use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub fn LogConsoleSkeleton() -> Element {
    rsx! {
        div { class: "log-viewer min-w-0 grid gap-3.5",
            div { class: "flex items-center justify-between gap-4",
                Skeleton { class: "h-2.5 w-32 max-w-[35%]" }
                div { class: "flex gap-1", for index in 0..5 { Skeleton { key: "{index}", class: "size-7 rounded" } } }
            }
            div { class: "flex flex-wrap items-end gap-2.5",
                Skeleton { class: "h-8 min-w-48 flex-1 mobile:basis-full" }
                for index in 0..3 {
                    div { key: "{index}", class: "grid gap-1.5 min-w-0 mobile:flex-1",
                        Skeleton { class: "h-2 w-10" }
                        Skeleton { class: "h-8 w-28 mobile:w-full" }
                    }
                }
            }
            div { class: "overflow-hidden rounded-[7px] border border-[#303030] bg-[#0c0c0c] [--panel:#0c0c0c] [--muted:#5a5a5a] [--foreground:#929292] [&_.ui-skeleton]:rounded-sm",
                div { class: "flex items-center justify-between border-b border-[#303030] bg-[#181818] px-3 py-3",
                    Skeleton { class: "h-2.5 w-24" }
                    Skeleton { class: "h-2.5 w-36 max-w-[40%]" }
                }
                div { class: "h-[clamp(220px,_48dvh,_580px)] max-h-[58dvh] min-h-40 overflow-hidden [--log-columns:minmax(140px,_190px)_58px_minmax(80px,_120px)_minmax(0,_1fr)] [@media(width<=900px)]:[--log-columns:minmax(100px,_145px)_52px_80px_minmax(0,_1fr)] mobile:[--log-columns:minmax(0,_1fr)_54px_minmax(0,_1fr)]",
                    div { class: "grid grid-cols-[var(--log-columns)] gap-3 border-b border-[#252525] bg-[#151515] px-3 py-2.5 mobile:gap-2",
                        for index in 0..4 { Skeleton { key: "{index}", class: if index == 3 { "h-2 w-16 mobile:hidden" } else { "h-2 w-10 max-w-full" } } }
                    }
                    for row in 0..12 {
                        div { key: "{row}", class: "grid grid-cols-[var(--log-columns)] items-center gap-3 border-b border-[#1c1c1c] px-3 py-3 mobile:gap-2",
                            Skeleton { class: "h-2 w-4/5" }
                            Skeleton { class: "h-3.5 w-10" }
                            Skeleton { class: "h-2 w-3/5 mobile:ml-auto" }
                            Skeleton { class: match row % 4 { 0 => "h-2 w-3/5 mobile:col-span-full", 1 => "h-2 w-5/6 mobile:col-span-full", 2 => "h-2 w-2/5 mobile:col-span-full", _ => "h-2 w-2/3 mobile:col-span-full" } }
                        }
                    }
                }
            }
            div { class: "flex justify-between gap-4",
                Skeleton { class: "h-2 w-48 max-w-[55%]" }
                Skeleton { class: "h-2 w-24 max-w-[30%]" }
            }
        }
    }
}
