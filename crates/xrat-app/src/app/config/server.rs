use serde::{Deserialize, Serialize};

use super::{SecretString, defaults};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub key: Option<SecretString>,
    pub web: WebSettings,
    pub pac_enabled: bool,
    pub pac_allowed_hosts: Vec<String>,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_SERVER_ENABLED,
            host: defaults::DEFAULT_SERVER_HOST.to_string(),
            port: defaults::DEFAULT_SERVER_PORT,
            key: None,
            web: WebSettings::default(),
            pac_enabled: defaults::DEFAULT_SERVER_PAC_ENABLED,
            pac_allowed_hosts: defaults::DEFAULT_SERVER_PAC_ALLOWED_HOSTS
                .iter()
                .map(|host| host.to_string())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct WebSettings {
    pub enabled: bool,
    pub admin_username: String,
    pub admin_password_hash: Option<SecretString>,
    pub public_url: Option<String>,
    pub transport: WebTransport,
    pub certificate: Option<std::path::PathBuf>,
    pub private_key: Option<std::path::PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebTransport {
    #[default]
    LocalHttp,
    ProxyHttps,
    DirectHttps,
}

impl WebSettings {
    pub fn secure(&self) -> bool {
        self.transport != WebTransport::LocalHttp
    }

    pub fn origin(&self, host: &str, port: u16) -> Result<String, String> {
        if self.transport == WebTransport::LocalHttp
            && !host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
        {
            return Err(
                "web local_http requires a loopback bind; use proxy_https or direct_https".into(),
            );
        }
        if let Some(public_url) = &self.public_url {
            let url = url::Url::parse(public_url).map_err(|_| "invalid web public_url")?;
            if !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
                || url.path() != "/"
                || url.host_str().is_none()
                || !matches!(url.scheme(), "http" | "https")
                || (self.secure() && url.scheme() != "https")
                || (!self.secure() && url.scheme() != "http")
            {
                return Err("web public_url must be an origin URL matching the transport".into());
            }
            return Ok(url.origin().ascii_serialization());
        }
        if self.transport == WebTransport::ProxyHttps {
            return Err("proxy_https requires web.public_url".into());
        }
        let host = host
            .parse::<std::net::IpAddr>()
            .map_err(|_| "web bind host must be an IP address")?;
        if !self.secure() && !host.is_loopback() {
            return Err(
                "web local_http requires a loopback bind; use proxy_https or direct_https".into(),
            );
        }
        let display_host = if host.is_unspecified() || host.is_loopback() {
            "localhost".into()
        } else {
            match host {
                std::net::IpAddr::V6(ip) => format!("[{ip}]"),
                _ => host.to_string(),
            }
        };
        Ok(format!(
            "{}://{display_host}:{port}",
            if self.secure() { "https" } else { "http" }
        ))
    }
}
