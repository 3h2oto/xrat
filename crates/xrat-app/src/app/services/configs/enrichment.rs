use crate::app::config::AppConfig;
use crate::app::context::RuntimePaths;
use crate::app::read_models::ConfigDetail;

/// Fill endpoint geolocation fields on joined rows when GeoIP is enabled.
///
/// Shared by CLI, HTTP, and TUI so location enrichment behaves the same
/// everywhere. Failures are silent by design: enrichment is best-effort.
pub async fn enrich_endpoint_locations(
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
    configs: &mut [ConfigDetail],
) {
    if !app_config.testing.geoip.enabled || !configs.iter().any(needs_location_enrichment) {
        return;
    }

    let Ok(lookup) = crate::app::geoip_backend::build_lookup_chain(app_config, runtime_paths)
    else {
        return;
    };

    for row in configs
        .iter_mut()
        .filter(|row| needs_location_enrichment(row))
    {
        let meta = xrat_support::geoip::enrich_address(&row.summary.address, lookup.as_ref()).await;
        if !meta.has_lookup_metadata() {
            continue;
        }
        if let Some(location) = meta.location {
            row.endpoint_location.location = Some(location);
        }
        if let Some(country) = meta.country {
            row.endpoint_location.country = Some(country);
        }
        if let Some(asn) = meta.asn {
            row.endpoint_location.asn = Some(asn);
        }
    }
}

fn needs_location_enrichment(row: &ConfigDetail) -> bool {
    row.endpoint_location.location.is_none()
        || row.endpoint_location.country.is_none()
        || row.endpoint_location.asn.is_none()
}
