use super::{
    button::{Button, ButtonSize, ButtonVariant},
    tooltip::{Tooltip, TooltipContent},
};
use dioxus::prelude::*;

#[component]
pub fn IconButton(
    label: String,
    #[props(into, optional)] class: Option<String>,
    #[props(default)] variant: ButtonVariant,
    #[props(default = ButtonSize::IconSm)] size: ButtonSize,
    #[props(default)] disabled: bool,
    #[props(into, optional)] button_type: Option<String>,
    #[props(into, optional)] href: Option<String>,
    #[props(into, optional)] download: Option<String>,
    #[props(optional)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let tooltip_id = use_hook(|| crate::hooks::use_random::use_random_id_for("icon-action"));
    rsx! {
        Tooltip { class: "icon-action", tabindex: if disabled { Some("0") } else { None },
            Button {
                class,
                variant,
                size,
                disabled,
                button_type,
                href,
                download,
                onclick,
                aria_label: label.clone(),
                aria_describedby: tooltip_id.clone(),
                attributes,
                {children}
                span { class: "sr-only", "{label}" }
            }
            TooltipContent { id: tooltip_id, "{label}" }
        }
    }
}
