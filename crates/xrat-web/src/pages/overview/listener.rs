use super::details::endpoint;
use crate::ui::{
    button::{Button, ButtonVariant},
    tooltip::{Tooltip, TooltipContent},
};
use dioxus::prelude::*;
use icons::Copy;
use xrat_web::types::RuntimeInbound;

#[component]
pub fn ListenerLink(inbound: RuntimeInbound) -> Element {
    let target_id = use_hook(|| crate::hooks::use_random::use_random_id_for("listener-url"));
    let Some(url) = listener_url(&inbound) else {
        return rsx! {
            code { {endpoint(&inbound.host, i64::from(inbound.port))} }
        };
    };
    rsx! {
        Tooltip { class: "runtime-listener-copy",
            Button {
                variant: ButtonVariant::Ghost,
                class: "runtime-listener-button",
                aria_label: format!("Copy {url}"),
                "data-copy-textarea": target_id.clone(),
                code { id: target_id, "{url}" }
                Copy {}
                span { class: "sr-only", "Copy listener URL" }
            }
            TooltipContent { "Click to copy listener URL" }
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
    Some(format!(
        "{scheme}://{}",
        endpoint(&inbound.host, i64::from(inbound.port))
    ))
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
            Some("ss://[fd00::10]:18200")
        );
        inbound.name = "Unknown".into();
        assert!(listener_url(&inbound).is_none());
    }
}
