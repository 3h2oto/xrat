use xrat_sdk::config::parse_link;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = std::env::args().nth(1).unwrap_or_else(|| {
        "vless://11111111-1111-1111-1111-111111111111@example.com:443#edge".into()
    });
    let node = parse_link(&link)?.ok_or("link contains no node")?;
    println!("{}", serde_json::to_string_pretty(&node)?);
    Ok(())
}
