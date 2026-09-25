use crate::app::config::AppConfig;
use crate::app::context::RuntimePaths;
use crate::db::record::ConfigWithLatestTest;

/// Fill endpoint geolocation fields on joined rows when GeoIP is enabled.
///
/// Shared by CLI, HTTP, and TUI so location enrichment behaves the same
/// everywhere. Failures are silent by design: enrichment is best-effort.
pub async fn enrich_endpoint_locations(
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
    configs: &mut [ConfigWithLatestTest],
) {
    if !app_config.testing.geoip.enabled || !configs.iter().any(needs_location_enrichment) {
        return;
    }

    let Ok(lookup) = crate::support::geoip::build_lookup_chain(app_config, runtime_paths) else {
        return;
    };

    for row in configs
        .iter_mut()
        .filter(|row| needs_location_enrichment(row))
    {
        let meta =
            crate::support::geoip::enrich_address(&row.config.address, lookup.as_ref()).await;
        if !meta.has_lookup_metadata() {
            continue;
        }
        if let Some(location) = meta.location {
            row.dial_endpoint_location = Some(location);
        }
        if let Some(country) = meta.country {
            row.dial_endpoint_country = Some(country);
        }
        if let Some(asn) = meta.asn {
            row.dial_endpoint_asn = Some(asn);
        }
    }
}

fn needs_location_enrichment(row: &ConfigWithLatestTest) -> bool {
    row.dial_endpoint_location.is_none()
        || row.dial_endpoint_country.is_none()
        || row.dial_endpoint_asn.is_none()
}
