//! Node and import-source fixtures shared by application tests.

use xrat_db::{ImportSource, SourceKind};
use xrat_model::{Node, Protocol};

/// A minimal raw-text import source for tests.
pub fn test_source() -> ImportSource {
    ImportSource {
        kind: SourceKind::RawText,
        value: "test".to_string(),
        name: Some("test".to_string()),
    }
}

/// A VLESS/TLS node pointing at `address`.
pub fn test_node(address: &str) -> Node {
    test_node_with(address, "test")
}

/// A VLESS/TLS node with an explicit display name.
pub fn test_node_with(address: &str, name: &str) -> Node {
    Node {
        protocol: Protocol::Vless,
        address: address.to_string(),
        port: 443,
        username: None,
        uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some(address.to_string()),
        host: None,
        path: None,
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!(
            "vless://00000000-0000-0000-0000-000000000000@{address}:443?security=tls#{name}"
        ),
    }
}
