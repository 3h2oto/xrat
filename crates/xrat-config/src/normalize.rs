use xrat_model::Node;

pub fn normalize(node: &mut Node) {
    if node.network.is_empty() {
        node.network = "tcp".to_string();
    }

    if node.network == "ws" {
        if node.host.is_none() {
            node.host = node.sni.clone();
        }
        if node.path.is_none() {
            node.path = Some("/".to_string());
        }
    }

    if node.network == "grpc" && node.path.is_none() {
        node.path = Some("/".to_string());
    }

    if matches!(node.tls.as_deref(), Some("")) {
        node.tls = None;
    }

    if let Some(extensions) = &mut node.extensions {
        for key in [
            "support-x25519mlkem768",
            "support_x25519mlkem768",
            "supportX25519Mlkem768",
            "supportX25519mlkem768",
        ] {
            extensions.remove(key);
        }
        if extensions.is_empty() {
            node.extensions = None;
        }
    }
}
