use std::time::Duration;
use xrat_sdk::{
    config::parse_link,
    engines::{singbox, xray},
    prober::tcp_check,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = parse_link("vless://11111111-1111-1111-1111-111111111111@example.com:443#edge")?
        .ok_or("expected a node")?;
    let normalized = serde_json::to_value(&node)?;
    assert_eq!(normalized["address"], "example.com");
    let xray = xray::generate_runtime_config(&node, 1080, Some(8080))?;
    let singbox = singbox::generate_singbox_runtime_config(
        &node,
        vec![singbox::SingboxInbound::socks(
            "socks-in",
            "127.0.0.1",
            1080,
            None,
        )],
        None,
        None,
    )?;
    assert_eq!(
        serde_json::to_value(&xray)?["outbounds"][0]["protocol"],
        "vless"
    );
    assert_eq!(
        serde_json::to_value(&singbox)?["outbounds"][0]["type"],
        "vless"
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let result = tcp_check(
        "127.0.0.1",
        listener.local_addr()?.port(),
        Duration::from_secs(1),
    )
    .await;
    assert!(result.success, "{:?}", result.failure_reason);
    println!("SDK consumer: parsing, node JSON, Xray JSON, sing-box JSON and TCP passed");
    Ok(())
}
