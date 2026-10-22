use crate::app::{
    AppError, Result,
    config::{ServerSettings, WebTransport},
};

pub(crate) fn frontend_pac_url(server: &ServerSettings) -> Result<String> {
    if !server.enabled {
        return Err(AppError::InvalidArgument(
            "API server is disabled; enable [server] before using PAC".into(),
        ));
    }
    if !server.pac_enabled {
        return Err(AppError::InvalidArgument(
            "PAC serving is disabled; set [server].pac_enabled = true before using PAC".into(),
        ));
    }
    let host = match server.host.as_str() {
        "" | "0.0.0.0" => "127.0.0.1",
        "::" => "::1",
        host => host,
    };
    let origin = match server.web.transport {
        WebTransport::LocalHttp if server.web.public_url.is_none() => format!(
            "http://{}",
            xrat_support::net::format_url_authority(host, server.port)
        ),
        WebTransport::LocalHttp | WebTransport::DirectHttps | WebTransport::ProxyHttps => server
            .web
            .origin(host, server.port)
            .map_err(AppError::InvalidArgument)?,
    };
    Ok(format!("{origin}/proxy.pac"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pac_url_respects_frontend_tls_and_requires_valid_proxy_origin() {
        let mut server = ServerSettings {
            enabled: true,
            pac_enabled: true,
            port: 18203,
            ..Default::default()
        };
        server.web.transport = WebTransport::DirectHttps;
        assert_eq!(
            frontend_pac_url(&server).unwrap(),
            "https://localhost:18203/proxy.pac"
        );
        server.web.transport = WebTransport::ProxyHttps;
        assert!(frontend_pac_url(&server).is_err());
        server.web.public_url = Some("https://operator.example".into());
        assert_eq!(
            frontend_pac_url(&server).unwrap(),
            "https://operator.example/proxy.pac"
        );
        server.web.public_url = Some("https://operator.example/private?token=unsafe".into());
        assert!(frontend_pac_url(&server).is_err());
    }
}
