use crate::app::services::desktop_proxy::{Backend, DesktopProxy, windows::WinInet};
use crate::app::services::proxy_pac::ProxyService;
use crate::app::{AppError, Result};
use crate::cli::ProxyDesktopAction;

pub(super) async fn run(context: &ProxyService, action: &ProxyDesktopAction) -> Result<()> {
    if xrat_support::windows::current_user_sid()? == "S-1-5-18" {
        return Err(AppError::InvalidArgument("desktop proxy settings require the interactive user's session; a LocalSystem service cannot change its own HKCU on the user's behalf".into()));
    }
    let directory = xrat_support::platform::xdg_data_home()
        .ok_or_else(|| {
            AppError::InvalidArgument("could not resolve the user data directory".into())
        })?
        .join("xrat");
    let backend = WinInet;
    let service = DesktopProxy::open(&backend, &directory)?;
    let desktop = match action {
        ProxyDesktopAction::Enable(args) => args.desktop,
        ProxyDesktopAction::Disable(args) => args.desktop,
        ProxyDesktopAction::Status(args) => args.desktop,
        ProxyDesktopAction::Toggle(args) => args.desktop,
    };
    if desktop.is_some() {
        return Err(AppError::InvalidArgument(
            "--desktop is a Linux desktop override; omit it on Windows".into(),
        ));
    }
    match action {
        ProxyDesktopAction::Status(_) => {}
        ProxyDesktopAction::Disable(_) => service.disable()?,
        ProxyDesktopAction::Toggle(_) if service.status()? == "XRAT-managed" => {
            service.disable()?
        }
        ProxyDesktopAction::Toggle(_) if service.status()? != "disabled" => {
            return Err(AppError::InvalidArgument("desktop proxy belongs to another application; use explicit enable to save and replace it".into()));
        }
        ProxyDesktopAction::Enable(_) | ProxyDesktopAction::Toggle(_) => {
            let pac = match action {
                ProxyDesktopAction::Enable(args) => args.pac,
                ProxyDesktopAction::Toggle(args) => args.pac,
                _ => false,
            };
            let mut settings = backend.read()?;
            if pac {
                settings.flags = 1 | 4;
                settings.pac = Some(super::pac_url(context)?);
                settings.server = None;
            } else {
                let active = super::super::resolve_active_endpoints(context).await?;
                let mut endpoints = Vec::new();
                if let Some((host, port)) = active.http {
                    let endpoint = xrat_support::net::format_host_port(
                        super::super::loopback_host(&host),
                        port,
                    );
                    endpoints.push(format!("http={endpoint}"));
                    endpoints.push(format!("https={endpoint}"));
                }
                if let Some((host, port)) = active.socks {
                    endpoints.push(format!(
                        "socks={}",
                        xrat_support::net::format_host_port(
                            super::super::loopback_host(&host),
                            port
                        )
                    ));
                }
                if endpoints.is_empty() {
                    return Err(AppError::InvalidArgument(
                        "no active HTTP or SOCKS inbound".into(),
                    ));
                }
                settings.flags = 1 | 2;
                settings.server = Some(endpoints.join(";"));
                settings.pac = None;
            }
            service.enable(settings)?;
        }
    }
    println!(
        "{}",
        crate::app::commands::output::format_kv(
            Some("Desktop proxy (Windows)"),
            &[("status", service.status()?.into())],
            crate::app::commands::output::color_enabled()
        )
    );
    Ok(())
}
