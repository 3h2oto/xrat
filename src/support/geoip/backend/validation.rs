use crate::app::config::{GeoIpBackend, GeoIpTestSettings};

use super::GeoIpError;

pub(crate) fn validate_geoip_settings(settings: &GeoIpTestSettings) -> Result<(), GeoIpError> {
    if settings.backend == GeoIpBackend::None {
        return Err(GeoIpError::InvalidSettings(
            "[testing.geoip].backend cannot be 'none'".to_string(),
        ));
    }

    if settings.backend != GeoIpBackend::Chain && settings.fallback != GeoIpBackend::None {
        return Err(GeoIpError::InvalidSettings(
            "[testing.geoip].fallback requires backend = 'chain'".to_string(),
        ));
    }

    if settings.backend == GeoIpBackend::Chain && settings.fallback == GeoIpBackend::None {
        return Err(GeoIpError::InvalidSettings(
            "[testing.geoip].fallback is required when backend = 'chain'".to_string(),
        ));
    }

    if settings.backend == settings.fallback && settings.fallback != GeoIpBackend::None {
        return Err(GeoIpError::InvalidSettings(
            "[testing.geoip].fallback must differ from backend".to_string(),
        ));
    }

    if settings.backend == GeoIpBackend::Chain
        && !matches!(
            settings.fallback,
            GeoIpBackend::IpWhois | GeoIpBackend::IpApi
        )
    {
        return Err(GeoIpError::InvalidSettings(
            "[testing.geoip].fallback must be ipwhois or ip-api when backend = 'chain'".to_string(),
        ));
    }

    if settings.cache.enabled && (settings.cache.ttl_secs == 0 || settings.cache.max_entries == 0) {
        return Err(GeoIpError::InvalidSettings(
            "[testing.geoip.cache] ttl_secs and max_entries must be positive when cache is enabled"
                .to_string(),
        ));
    }

    Ok(())
}
