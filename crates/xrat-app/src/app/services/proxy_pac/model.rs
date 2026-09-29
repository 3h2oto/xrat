use crate::app::config::RoutingSettings;

/// Local proxy endpoints used to render a PAC file. Hosts/ports are non-secret
/// and safe to expose; Shadowsocks credentials are intentionally excluded.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PacEndpoints {
    pub http: Option<(String, u16)>,
    pub socks: Option<(String, u16)>,
}

/// Routing rules translated into PAC decision lists.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PacRules {
    pub direct_domains: Vec<String>,
    pub direct_cidrs: Vec<String>,
    pub block_domains: Vec<String>,
    pub block_cidrs: Vec<String>,
}

impl PacRules {
    pub fn from_routing(routing: &RoutingSettings) -> Self {
        Self {
            direct_domains: routing.direct.domain.clone(),
            direct_cidrs: routing.direct.ip.clone(),
            block_domains: routing.block.domain.clone(),
            block_cidrs: routing.block.ip.clone(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.direct_domains.is_empty()
            && self.direct_cidrs.is_empty()
            && self.block_domains.is_empty()
            && self.block_cidrs.is_empty()
    }
}
