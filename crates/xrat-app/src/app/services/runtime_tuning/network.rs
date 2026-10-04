use super::prelude::*;

/// Resolve the inbound listen address when `[runtime.network].listen_interface`
/// is set, returning the interface's address. Returns `None` when no interface
/// is configured so callers fall back to the per-inbound host.
pub(crate) fn resolve_listen_interface_addr(
    runtime: &RuntimeSettings,
) -> crate::app::Result<Option<String>> {
    let interface = runtime.network.listen_interface.trim();
    if interface.is_empty() {
        return Ok(None);
    }
    xrat_support::net::interface_address(interface)
        .map(Some)
        .ok_or_else(|| {
            AppError::InvalidArgument(format!(
                "[runtime.network].listen_interface \"{interface}\" has no resolvable address"
            ))
        })
}

/// Translate the dual-form packets setting to Xray's `packets` value: the
/// `tlshello` keyword, or a `min-max` range when `packets_mode = "range"`.
pub(crate) fn fragment_packets(runtime: &RuntimeSettings) -> String {
    if runtime.fragment.packets_mode.trim() == "range" {
        format_range(runtime.fragment.packets)
    } else {
        "tlshello".to_string()
    }
}

pub(crate) fn format_range(range: [u32; 2]) -> String {
    format!("{}-{}", range[0], range[1])
}

pub(crate) fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
