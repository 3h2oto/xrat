use crate::app::commands::output;
use crate::app::commands::runtime_output::format_inbound_endpoint;
use crate::app::services::proxy_pac::ProxyService;

use super::{loopback_host, resolve_active_endpoints};

pub(super) async fn run(context: &ProxyService, json: bool) -> crate::app::Result<()> {
    let active = resolve_active_endpoints(context).await?;

    let mut rows: Vec<(&str, String)> = Vec::new();
    if let Some((host, port)) = &active.http {
        rows.push((
            "HTTP",
            format!(
                "http://{}",
                format_inbound_endpoint(host, *port).replace('%', "%25")
            ),
        ));
    }
    if let Some((host, port)) = &active.socks {
        rows.push((
            "SOCKS5",
            format!(
                "socks5://{}",
                format_inbound_endpoint(host, *port).replace('%', "%25")
            ),
        ));
    }
    if let Some((host, port)) = &active.shadowsocks {
        // The runtime session does not persist Shadowsocks credentials, so emit
        // the endpoint without a leaky partial ss:// URI.
        rows.push((
            "Shadowsocks",
            format!(
                "{} (credentials not shown)",
                format_inbound_endpoint(host, *port)
            ),
        ));
    }

    let server = context.server()?;
    let pac_url = (server.enabled && server.pac_enabled)
        .then(|| crate::app::services::proxy_pac::frontend_pac_url(&server))
        .transpose()?;
    if let Some(url) = &pac_url {
        rows.push(("PAC URL", url.clone()));
    }

    if json {
        let payload = serde_json::json!({
            "endpoints": {
                "http": active.http.as_ref().map(|(h, p)| super::http_proxy_url(h, *p)),
                "socks5": active.socks.as_ref().map(|(h, p)| super::socks_proxy_url(h, *p)),
                "shadowsocks": active.shadowsocks.as_ref().map(|(h, p)| xrat_support::net::format_host_port(loopback_host(h), *p)),
                "pac": pac_url,
            },
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!(
            "{}",
            output::empty_message(
                "No active proxy endpoints. Start a runtime with `xrat connect <id>`."
            )
        );
        return Ok(());
    }

    let kv: Vec<(&str, String)> = rows;
    println!(
        "{}",
        output::format_kv(Some("Proxy info"), &kv, output::color_enabled())
    );
    Ok(())
}
