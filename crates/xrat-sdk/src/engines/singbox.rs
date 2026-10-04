//! Typed sing-box configuration generation. Generating a config does not launch sing-box.

pub use xrat_engines::singbox::{
    SingboxCacheFile, SingboxClashApi, SingboxConfig, SingboxDnsConfig, SingboxExperimental,
    SingboxInbound, SingboxInboundUser, SingboxLogConfig, SingboxRoute, SingboxRouteList,
    SingboxRoutingOptions, generate_singbox_probe_config, generate_singbox_runtime_config,
    generate_singbox_runtime_config_with_dns,
};
