use super::*;

pub(in super::super) fn test_node(name: &str) -> Node {
    Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("uuid-123".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: Some("/socket".to_string()),
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!("vless://uuid-123@example.com:443?type=ws&security=tls#{name}"),
    }
}
