use super::*;

pub async fn run(context: &AppContext, args: &ProxyArgs) -> crate::app::Result<()> {
    run_service(&ProxyService::from_context(context), args).await
}

pub(crate) async fn run_service(
    context: &ProxyService,
    args: &ProxyArgs,
) -> crate::app::Result<()> {
    match &args.action {
        ProxyAction::Info(info_args) => endpoints::run(context, info_args.json).await,
        ProxyAction::Pac(pac_args) => match &pac_args.action {
            ProxyPacAction::Url(_) => pac::print_pac_url(context),
            ProxyPacAction::Print(_) => pac::print_pac_file(context).await,
        },
        ProxyAction::Shell(shell_args) => shell::run(context, &shell_args.action).await,
        ProxyAction::Desktop(desktop_args) => desktop::run(context, &desktop_args.action).await,
    }
}

pub(crate) async fn resolve_active_endpoints(
    context: &ProxyService,
) -> crate::app::Result<ActiveEndpoints> {
    context.endpoints().await
}

pub(crate) fn loopback_host(host: &str) -> &str {
    match host {
        "" | "0.0.0.0" => "127.0.0.1",
        "::" => "::1",
        _ => host,
    }
}

pub(crate) fn http_proxy_url(host: &str, port: u16) -> String {
    format!(
        "http://{}",
        xrat_support::net::format_url_authority(loopback_host(host), port)
    )
}

pub(crate) fn socks_proxy_url(host: &str, port: u16) -> String {
    format!(
        "socks5://{}",
        xrat_support::net::format_url_authority(loopback_host(host), port)
    )
}
