use crate::app::commands::output;
use crate::app::services::proxy_pac::ProxyService;

pub(super) fn print_pac_url(context: &ProxyService) -> crate::app::Result<()> {
    let server = context.server()?;
    if !server.enabled || !server.pac_enabled {
        let message = if !server.enabled {
            "API server is disabled; enable [server] and run `xrat serve` (or the daemon) to serve /proxy.pac."
        } else {
            "PAC serving is disabled; set [server].pac_enabled = true to serve /proxy.pac."
        };
        println!("{}", output::notice(message, output::color_enabled()));
        return Ok(());
    }

    println!(
        "{}",
        crate::app::services::proxy_pac::frontend_pac_url(&server)?
    );
    Ok(())
}

pub(super) async fn print_pac_file(context: &ProxyService) -> crate::app::Result<()> {
    print!("{}", context.pac().await?);
    Ok(())
}
