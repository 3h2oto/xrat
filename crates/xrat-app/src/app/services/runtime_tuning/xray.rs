use super::network::*;
use super::prelude::*;

/// Translate runtime tuning settings into outbound generation options. Routing
/// is added separately for managed sessions so probe configs remain proxy-only.
pub(crate) fn build_xray_gen_options(runtime: &RuntimeSettings) -> XrayGenOptions {
    let mux = runtime.mux.enabled.then(|| MuxOptions {
        concurrency: runtime.mux.concurrency,
        xudp_concurrency: runtime.mux.xudp_concurrency,
        xudp_proxy_udp443: runtime.mux.xudp_proxy_udp443.clone(),
    });
    let fragment = runtime.fragment.enabled.then(|| FragmentOptions {
        packets: fragment_packets(runtime),
        length: format_range(runtime.fragment.length),
        interval: format_range(runtime.fragment.interval),
    });

    XrayGenOptions {
        compatibility: match runtime.xray_compatibility {
            XrayCompatibilityPolicy::Prerelease => XrayCompatibilityTarget::PrereleaseV26_7_28,
            XrayCompatibilityPolicy::Auto | XrayCompatibilityPolicy::Stable => {
                XrayCompatibilityTarget::StableV26_3_27
            }
        },
        mux,
        fragment,
        interface: non_empty(&runtime.network.interface),
        mark: (runtime.network.mark != 0).then_some(runtime.network.mark),
        bind_address: non_empty(&runtime.network.bind_address),
        routing: None,
        dns: None,
    }
}

pub(crate) fn detect_xray_compatibility(
    policy: XrayCompatibilityPolicy,
    binary_path: &Path,
) -> XrayCompatibilityTarget {
    detect_xray_compatibility_with_spawner(
        policy,
        binary_path,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
    )
}

pub(crate) fn detect_xray_compatibility_with_spawner(
    policy: XrayCompatibilityPolicy,
    binary_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> XrayCompatibilityTarget {
    match policy {
        XrayCompatibilityPolicy::Stable => XrayCompatibilityTarget::StableV26_3_27,
        XrayCompatibilityPolicy::Prerelease => XrayCompatibilityTarget::PrereleaseV26_7_28,
        XrayCompatibilityPolicy::Auto => Command::with_spawner(binary_path, spawner.clone())
            .arg("version")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
            .filter(|version| version.contains("26.7.28"))
            .map(|_| XrayCompatibilityTarget::PrereleaseV26_7_28)
            .unwrap_or(XrayCompatibilityTarget::StableV26_3_27),
    }
}

pub(crate) fn apply_xray_dns_options(
    options: &mut XrayGenOptions,
    dns: &DnsSettings,
) -> crate::app::Result<()> {
    if dns == &DnsSettings::default() {
        return Ok(());
    }

    let query_strategy = match dns.query_strategy.as_str() {
        "UseIP" | "UseIPv4" | "UseIPv6" | "UseSystem" => dns.query_strategy.clone(),
        other => {
            return Err(AppError::InvalidArgument(format!(
                "[dns].query_strategy must be UseIP, UseIPv4, UseIPv6, or UseSystem; got \"{other}\""
            )));
        }
    };

    let mut servers = Vec::with_capacity(dns.servers.len());
    for server in &dns.servers {
        let server = server.trim();
        if server.is_empty() {
            return Err(AppError::InvalidArgument(
                "[dns].servers cannot contain an empty server".to_string(),
            ));
        }
        servers.push(server.to_string());
    }

    let hosts = dns
        .hosts
        .iter()
        .map(|(host, value)| {
            let value = match value {
                DnsHostValue::One(value) => XrayDnsHostValue::One(value.clone()),
                DnsHostValue::Many(value) => XrayDnsHostValue::Many(value.clone()),
            };
            (host.clone(), value)
        })
        .collect();

    options.dns = Some(XrayDnsConfig {
        servers,
        hosts,
        query_strategy,
        use_system_hosts: dns.use_system_hosts,
        disable_cache: dns.disable_cache,
        disable_fallback: dns.disable_fallback,
        enable_parallel_query: dns.enable_parallel_query,
    });
    Ok(())
}

pub(crate) fn apply_xray_routing_options(options: &mut XrayGenOptions, routing: &RoutingSettings) {
    options.routing = Some(XrayRoutingOptions {
        domain_strategy: routing.domain_strategy.clone(),
        direct: xray_route_list(&routing.direct),
        block: xray_route_list(&routing.block),
    });
}

fn xray_route_list(routes: &RouteList) -> XrayRouteList {
    XrayRouteList {
        domain: routes.domain.clone(),
        ip: routes.ip.clone(),
        geosite: routes.geosite.clone(),
        geoip: routes.geoip.clone(),
    }
}
