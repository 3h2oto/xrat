use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::services::proxy_pac::{PacRules, active_pac};

pub(super) fn print_pac_url(context: &AppContext) {
    let server = &context.app_config.server;
    if !server.enabled || !server.pac_enabled {
        let message = if !server.enabled {
            "API server is disabled; enable [server] and run `xrat serve` (or the daemon) to serve /proxy.pac."
        } else {
            "PAC serving is disabled; set [server].pac_enabled = true to serve /proxy.pac."
        };
        println!("{}", output::notice(message, output::color_enabled()));
        return;
    }

    // PAC consumers should fetch over loopback; show 127.0.0.1 for a wildcard
    // bind rather than an unusable 0.0.0.0 URL.
    let host = if server.host == "0.0.0.0" || server.host.is_empty() {
        "127.0.0.1"
    } else {
        server.host.as_str()
    };
    println!("http://{host}:{}/proxy.pac", server.port);
}

pub(super) async fn print_pac_file(context: &AppContext) -> crate::app::Result<()> {
    print!(
        "{}",
        active_pac(
            &context.db,
            &PacRules::from_routing(&context.app_config.routing)
        )
        .await?
    );
    Ok(())
}
