mod network;
mod prelude;
mod singbox;
mod xray;

#[cfg(test)]
mod tests;

pub(crate) use network::resolve_listen_interface_addr;
pub(crate) use singbox::{build_singbox_dns_options, build_singbox_routing_options};
pub(crate) use xray::{
    apply_xray_dns_options, apply_xray_routing_options, build_xray_gen_options,
    detect_xray_compatibility, detect_xray_compatibility_with_spawner,
    ensure_xray_tun_supported_with_spawner, xray_binary_version_with_spawner,
};
