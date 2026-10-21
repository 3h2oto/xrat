use super::details::endpoint;
use crate::ui::{
    button::{Button, ButtonVariant},
    tooltip::{Tooltip, TooltipContent},
};
use dioxus::prelude::*;
use icons::Copy;
use xrat_web::types::RuntimeInbound;

pub(super) fn pac_url(available: bool) -> Option<String> {
    if !available {
        return None;
    }
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .map(|origin| format!("{origin}/proxy.pac"))
}

#[component]
pub fn ListenerLink(inbound: RuntimeInbound) -> Element {
    let Some(url) = listener_url(&inbound) else {
        return rsx! {
            code { {endpoint(&inbound.host, i64::from(inbound.port))} }
        };
    };
    rsx! {
        CopyUrl { url }
    }
}

#[component]
pub(super) fn CopyUrl(url: String) -> Element {
    let feedback = crate::components::clipboard::use_copy_feedback();
    let target_id = use_hook(|| crate::hooks::use_random::use_random_id_for("listener-url"));
    rsx! {
        Tooltip { class: "runtime-listener-copy flex items-center min-w-0 max-w-full m-0",
            Button {
                variant: ButtonVariant::Ghost,
                class: "runtime-listener-button max-w-full h-auto min-h-6.5 justify-start whitespace-normal text-left py-[3px] px-0 gap-[7px] [&_code]:min-w-0 [&_>_svg]:size-3 [&_>_svg]:text-muted-foreground",
                aria_label: format!("Copy {url}"),
                onclick: {
                    let target_id = target_id.clone();
                    move |event| feedback.copy_target(&target_id, event)
                },
                code { id: target_id, "{url}" }
                Copy {}
                span { class: "sr-only", {feedback.label("Copy URL")} }
            }
            TooltipContent { {feedback.label("Click to copy URL")} }
        }
    }
}

fn listener_url(inbound: &RuntimeInbound) -> Option<String> {
    let scheme = match inbound.name.as_str() {
        "SOCKS" => "socks5",
        "HTTP" => "http",
        "Shadowsocks" => "ss",
        _ => return None,
    };
    let address = endpoint(&inbound.host, i64::from(inbound.port));
    let suffix = if scheme == "ss" { "/" } else { "" };
    Some(format!("{scheme}://{address}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener_urls_include_protocol_and_bracket_ipv6() {
        let mut inbound = RuntimeInbound {
            name: "SOCKS".into(),
            host: "192.168.1.168".into(),
            port: 18200,
            reachable: None,
        };
        assert_eq!(
            listener_url(&inbound).as_deref(),
            Some("socks5://192.168.1.168:18200")
        );
        inbound.name = "HTTP".into();
        inbound.host = "fd00::10".into();
        assert_eq!(
            listener_url(&inbound).as_deref(),
            Some("http://[fd00::10]:18200")
        );
        inbound.name = "Shadowsocks".into();
        assert_eq!(
            listener_url(&inbound).as_deref(),
            Some("ss://[fd00::10]:18200/")
        );
        inbound.name = "Unknown".into();
        assert!(listener_url(&inbound).is_none());
    }
}
