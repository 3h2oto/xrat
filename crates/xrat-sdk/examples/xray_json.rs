use xrat_sdk::{config::parse_link, engines::xray};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = std::env::args().nth(1).unwrap_or_else(|| {
        "vless://11111111-1111-1111-1111-111111111111@example.com:443#edge".into()
    });
    let node = parse_link(&link)?.ok_or("link contains no node")?;
    let options = xray::XrayGenOptions::default();
    let config = xray::generate_runtime_config_for_inbounds_with_options(
        &node,
        Some(("127.0.0.1", 1080, true)),
        Some(("127.0.0.1", 8080)),
        &options,
    )?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
