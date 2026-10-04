use std::{path::Path, time::Duration};
use xrat_sdk::{
    config::parse_link,
    engines::xray::XrayGenOptions,
    prober::{AcceptedHttpStatuses, ProbeEngineKind, real_delay_check},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [engine, binary, link, url] = args.as_slice() else {
        return Err(
            "usage: real_delay <xray|sing-box> <binary-path> <proxy-link> <test-url>".into(),
        );
    };
    let engine = match engine.as_str() {
        "xray" => ProbeEngineKind::Xray,
        "sing-box" => ProbeEngineKind::Singbox,
        _ => return Err("engine must be xray or sing-box".into()),
    };
    let node = parse_link(link)?.ok_or("link contains no node")?;
    let result = real_delay_check(
        &node,
        url,
        engine,
        Path::new(binary),
        Duration::from_secs(5),
        Duration::from_secs(10),
        &XrayGenOptions::default(),
        &AcceptedHttpStatuses::default(),
        true,
    )
    .await;
    if !result.success {
        return Err(format!("Proxy probe failed: {:?}", result.failure_reason).into());
    }
    println!(
        "HTTP {:?}, latency {:?} ms",
        result.http_status, result.latency_ms
    );
    Ok(())
}
