use super::{PacEndpoints, PacRules, render_pac};

/// Active local proxy endpoints resolved from the running runtime session.
/// Hosts/ports only; never includes Shadowsocks credentials.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActiveEndpoints {
    pub http: Option<(String, u16)>,
    pub socks: Option<(String, u16)>,
    pub shadowsocks: Option<(String, u16)>,
}

pub async fn active_endpoints(db: &xrat_db::Database) -> crate::app::Result<ActiveEndpoints> {
    let Some(session) = db.get_running_runtime_session().await? else {
        return Ok(ActiveEndpoints::default());
    };

    Ok(ActiveEndpoints {
        http: endpoint(session.http_host, session.http_port),
        socks: endpoint(session.socks_host, session.socks_port),
        shadowsocks: endpoint(session.shadowsocks_host, session.shadowsocks_port),
    })
}

fn endpoint(host: Option<String>, port: Option<i64>) -> Option<(String, u16)> {
    match (host, port) {
        (Some(host), Some(port)) if port > 0 && port <= i64::from(u16::MAX) => {
            Some((host, port as u16))
        }
        _ => None,
    }
}

pub async fn active_pac(db: &xrat_db::Database, rules: &PacRules) -> crate::app::Result<String> {
    let active = active_endpoints(db).await?;
    Ok(render_pac(
        &PacEndpoints {
            http: active.http,
            socks: active.socks,
        },
        rules,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_or_incomplete_runtime_endpoints_are_omitted() {
        for port in [None, Some(-1), Some(0), Some(65536)] {
            assert_eq!(endpoint(Some("localhost".into()), port), None);
        }
        assert_eq!(endpoint(None, Some(1080)), None);
        for port in [1, 65535] {
            assert_eq!(
                endpoint(Some("localhost".into()), Some(port)),
                Some(("localhost".into(), port as u16))
            );
        }
    }
}
