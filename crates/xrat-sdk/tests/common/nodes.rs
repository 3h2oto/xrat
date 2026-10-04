use xrat_sdk::{config::parse_link, model::Node};

pub fn nodes() -> Vec<Node> {
    [
        "vless://11111111-1111-1111-1111-111111111111@127.0.0.1:443",
        "vmess://eyJ2IjogIjIiLCAiYWRkIjogIjEyNy4wLjAuMSIsICJwb3J0IjogIjQ0MyIsICJpZCI6ICIxMTExMTExMS0xMTExLTExMTEtMTExMS0xMTExMTExMTExMTEiLCAiYWlkIjogIjAiLCAibmV0IjogInRjcCIsICJzY3kiOiAiYXV0byIsICJ0bHMiOiAiIn0=",
        "trojan://password@127.0.0.1:443?sni=localhost",
        "ss://YWVzLTEyOC1nY206cGFzc3dvcmQ=@127.0.0.1:443",
        "http://127.0.0.1:443",
        "socks5://127.0.0.1:443",
        "hy2://password@127.0.0.1:443?sni=localhost",
    ]
    .into_iter()
    .map(|link| parse_link(link).unwrap().unwrap())
    .collect()
}
