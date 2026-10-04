use std::time::Duration;
use xrat_sdk::prober::tcp_check;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let result = tcp_check("127.0.0.1", address.port(), Duration::from_secs(1)).await;
    if !result.success {
        return Err(format!("TCP probe failed: {:?}", result.failure_reason).into());
    }
    println!("Local TCP probe succeeded: {:?} ms", result.latency_ms);
    Ok(())
}
