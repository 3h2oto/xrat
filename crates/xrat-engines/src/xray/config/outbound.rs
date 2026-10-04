use serde_json::json;

use super::XrayCompatibilityTarget;
use super::extensions::ExtensionResolver;
use super::stream::build_stream_settings;
use super::types::Outbound;
use xrat_model::{Node, Protocol};

pub(super) fn node_to_outbound(
    node: &Node,
    tag: &str,
    compatibility: XrayCompatibilityTarget,
) -> Result<Outbound, String> {
    let protocol = match node.protocol {
        Protocol::Ss => "shadowsocks",
        Protocol::Socks5 => "socks",
        Protocol::Hy2 => "hysteria",
        _ => node.protocol.as_str(),
    }
    .to_string();
    let mut extensions = ExtensionResolver::new(node);
    let settings = build_outbound_settings(node, &mut extensions)?;
    let stream_settings = build_stream_settings(node, &mut extensions, compatibility)?;
    extensions.finish()?;

    Ok(Outbound {
        tag: tag.to_string(),
        protocol,
        settings,
        stream_settings,
        mux: None,
    })
}

fn build_outbound_settings(
    node: &Node,
    extensions: &mut ExtensionResolver,
) -> Result<serde_json::Value, String> {
    match node.protocol {
        Protocol::Vless => {
            let uuid = node.uuid.as_ref().ok_or("vless requires uuid")?;
            let mut user = json!({
                "id": uuid,
                "encryption": "none"
            });
            if let Some(flow) = extensions.string("flow")?.filter(|value| !value.is_empty()) {
                user["flow"] = json!(flow);
            }
            if let Some(encryption) = extensions.string("encryption")? {
                user["encryption"] = json!(encryption);
            }
            Ok(json!({
                "vnext": [{
                    "address": node.address,
                    "port": node.port,
                    "users": [user]
                }]
            }))
        }
        Protocol::Vmess => {
            let uuid = node.uuid.as_ref().ok_or("vmess requires uuid")?;
            Ok(json!({
                "vnext": [{
                    "address": node.address,
                    "port": node.port,
                    "users": [{
                        "id": uuid,
                        "alterId": extensions.u64("aid")?.unwrap_or(0),
                        "security": extensions.alias_string("encryption", &["scy", "security"])?.unwrap_or_else(|| "auto".to_string())
                    }]
                }]
            }))
        }
        Protocol::Trojan => {
            let password = node.password.as_ref().ok_or("trojan requires password")?;
            let mut server = json!({
                "address": node.address,
                "port": node.port,
                "password": password
            });
            if let Some(flow) = extensions.string("flow")?.filter(|value| !value.is_empty()) {
                server["flow"] = json!(flow);
            }
            Ok(json!({"servers": [server]}))
        }
        Protocol::Ss => {
            let password = node
                .password
                .as_ref()
                .ok_or("shadowsocks requires password")?;
            let method = node.method.as_ref().ok_or("shadowsocks requires method")?;
            Ok(json!({
                "servers": [{
                    "address": node.address,
                    "port": node.port,
                    "method": method,
                    "password": password
                }]
            }))
        }
        Protocol::Socks5 => {
            let mut server = json!({
                "address": node.address,
                "port": node.port
            });
            if let Some(username) = &node.username
                && let Some(password) = &node.password
            {
                server["users"] = json!([{
                    "user": username,
                    "pass": password
                }]);
            }
            Ok(json!({
                "servers": [server]
            }))
        }
        Protocol::Http => {
            let mut server = json!({
                "address": node.address,
                "port": node.port
            });
            if let Some(username) = &node.username
                && let Some(password) = &node.password
            {
                server["users"] = json!([{
                    "user": username,
                    "pass": password
                }]);
            }
            Ok(json!({
                "servers": [server]
            }))
        }
        Protocol::Hy2 => Ok(json!({
            "version": 2,
            "address": node.address,
            "port": node.port
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_normalized_protocol_names_to_native_xray_names() {
        for (link, normalized, native) in [
            (
                "ss://YWVzLTEyOC1nY206cGFzc3dvcmQ=@127.0.0.1:443",
                Protocol::Ss,
                "shadowsocks",
            ),
            ("socks5://127.0.0.1:443", Protocol::Socks5, "socks"),
        ] {
            let node = xrat_config::parse_link(link).unwrap().unwrap();
            assert_eq!(node.protocol, normalized);
            let outbound =
                node_to_outbound(&node, "proxy", XrayCompatibilityTarget::default()).unwrap();
            assert_eq!(outbound.protocol, native);
            assert_eq!(node.protocol, normalized);
        }
    }
}
