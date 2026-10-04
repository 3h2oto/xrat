use super::*;

pub async fn run(context: &AppContext, args: &ProxyArgs) -> crate::app::Result<()> {
    match &args.action {
        ProxyAction::Info(info_args) => endpoints::run(context, info_args.json).await,
        ProxyAction::Pac(pac_args) => match &pac_args.action {
            ProxyPacAction::Url(_) => {
                pac::print_pac_url(context);
                Ok(())
            }
            ProxyPacAction::Print(_) => pac::print_pac_file(context).await,
        },
        ProxyAction::Shell(shell_args) => shell::run(context, &shell_args.action).await,
        ProxyAction::Desktop(desktop_args) => desktop::run(context, &desktop_args.action).await,
    }
}

pub(crate) async fn resolve_active_endpoints(
    context: &AppContext,
) -> crate::app::Result<ActiveEndpoints> {
    crate::app::services::proxy_pac::active_endpoints(&context.db).await
}

pub(crate) fn loopback_host(host: &str) -> &str {
    if host == "0.0.0.0" || host.is_empty() {
        "127.0.0.1"
    } else {
        host
    }
}

pub(crate) fn http_proxy_url(host: &str, port: u16) -> String {
    format!("http://{}:{port}", loopback_host(host))
}

pub(crate) fn socks_proxy_url(host: &str, port: u16) -> String {
    format!("socks5://{}:{port}", loopback_host(host))
}
