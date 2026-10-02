use crate::app::read_models::ConfigDetail;

/// Persisted host -> geo entries older than this are treated as cache misses and
/// re-resolved. Geo is stable, so a day keeps boots network-free while still
/// letting dead hosts recover.
const GEO_CACHE_TTL_SECS: i64 = 86_400;

/// Apply persisted host -> geo to rows still missing location, returning the
/// `(config_id, address)` rows that need a network lookup. A fresh cache entry
/// (even an empty one) suppresses re-resolution so unresolvable hosts are not
/// retried on every boot.
pub(super) async fn apply_geo_cache(
    context: &crate::app::context::AppContext,
    configs: &mut [ConfigDetail],
    clock: &dyn crate::app::ports::Clock,
) -> Vec<(xrat_model::ConfigId, String)> {
    let targets: Vec<(usize, xrat_model::ConfigId, String, String)> = configs
        .iter()
        .enumerate()
        .filter(|(_, config)| needs_location_enrichment(config))
        .filter_map(|(index, config)| {
            xrat_support::geoip::address_host(&config.summary.address).map(|host| {
                (
                    index,
                    config.summary.id,
                    config.summary.address.clone(),
                    host,
                )
            })
        })
        .collect();
    if targets.is_empty() {
        return Vec::new();
    }

    let mut hosts: Vec<String> = targets.iter().map(|(_, _, _, host)| host.clone()).collect();
    hosts.sort();
    hosts.dedup();

    let now = clock.now_unix_secs();
    let cached: std::collections::HashMap<String, xrat_db::GeoIpCacheRecord> = context
        .db
        .get_fresh_geoip_cache(&hosts, now - GEO_CACHE_TTL_SECS)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|record| (record.host.clone(), record))
        .collect();

    let mut pending = Vec::new();
    for (index, id, address, host) in targets {
        match cached.get(&host) {
            Some(record) => {
                if record.has_location() {
                    let location = &mut configs[index].endpoint_location;
                    if record.location.is_some() {
                        location.location = record.location.clone();
                    }
                    if record.country.is_some() {
                        location.country = record.country.clone();
                    }
                    if record.asn.is_some() {
                        location.asn = record.asn.clone();
                    }
                    location.fronting = None;
                }
            }
            None => pending.push((id, address)),
        }
    }
    pending
}

/// Time-to-live for in-session GeoIP lookups so repeated resolutions of the same
/// IP reuse the decoded mmdb result instead of re-reading the database.
const GEO_LOOKUP_TTL: std::time::Duration = std::time::Duration::from_secs(3600);

/// Upper bound on cached IP lookups kept in memory for a single session.
const GEO_LOOKUP_MAX_ENTRIES: usize = 8192;

/// Build the GeoIP lookup used for background location enrichment, wrapping the
/// local mmdb backend in an in-session [`CachedLookup`] so the lookup is built
/// once per session rather than rebuilt on every data load.
pub fn build_geo_lookup(
    app_config: &crate::app::config::AppConfig,
    runtime_paths: &crate::app::context::RuntimePaths,
) -> std::sync::Arc<xrat_support::geoip::CachedLookup> {
    let inner = std::sync::Arc::new(local_mmdb_lookup(app_config, runtime_paths));
    std::sync::Arc::new(xrat_support::geoip::CachedLookup::new(
        inner,
        GEO_LOOKUP_TTL,
        GEO_LOOKUP_MAX_ENTRIES,
    ))
}

fn local_mmdb_lookup(
    app_config: &crate::app::config::AppConfig,
    runtime_paths: &crate::app::context::RuntimePaths,
) -> xrat_support::geoip::LocalMmdbLookup {
    xrat_support::geoip::LocalMmdbLookup::new(
        crate::app::paths::mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.country_path,
            "GeoLite2-Country.mmdb",
        ),
        crate::app::paths::mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.city_path,
            "GeoLite2-City.mmdb",
        ),
        crate::app::paths::mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.asn_path,
            "GeoLite2-ASN.mmdb",
        ),
    )
}

fn needs_location_enrichment(config: &ConfigDetail) -> bool {
    if config.summary.address.trim().is_empty() {
        return false;
    }
    let geo = &config.endpoint_location;
    !(geo.country.is_some()
        || geo.asn.is_some()
        || geo
            .location
            .as_deref()
            .is_some_and(|label| !xrat_support::geoip::is_classified_placeholder(label)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enrichment_requires_a_nonempty_address_and_missing_real_geo() {
        let mut config = ConfigDetail::from_joined(
            &crate::app::services::test_support::sample_joined_row(1, true),
        );
        assert!(needs_location_enrichment(&config));
        config.endpoint_location.country = Some("NL".into());
        assert!(!needs_location_enrichment(&config));
        config.endpoint_location.country = None;
        config.endpoint_location.asn = Some("AS64512 TEST".into());
        assert!(!needs_location_enrichment(&config));
        config.endpoint_location.asn = None;
        config.summary.address = "   ".into();
        assert!(!needs_location_enrichment(&config));
        config.summary.address = "example.com".into();
        config.endpoint_location.location = Some("loopback_ipv4".into());
        assert!(needs_location_enrichment(&config));
        config.endpoint_location.location = Some("NL/Amsterdam".into());
        assert!(!needs_location_enrichment(&config));
    }
}
