use xrat_sdk::{config::parse_link, engines::singbox};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = std::env::args().nth(1).unwrap_or_else(|| {
        "vless://11111111-1111-1111-1111-111111111111@example.com:443#edge".into()
    });
    let node = parse_link(&link)?.ok_or("link contains no node")?;
    let inbounds = vec![
        singbox::SingboxInbound::socks("socks-in", "127.0.0.1", 1080, None),
        singbox::SingboxInbound::http("http-in", "127.0.0.1", 8080),
    ];
    let config = singbox::generate_singbox_runtime_config(&node, inbounds, None, None)?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
