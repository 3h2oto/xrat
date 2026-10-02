use std::sync::Arc;
use xrat_support::geoip::CachedLookup;

/// Bound on concurrent address resolutions so enrichment does not fan out one
/// DNS query per config serially.
const ENRICH_CONCURRENCY: usize = 64;

/// Per-address cap so an unresolvable or slow host cannot stall enrichment.
const ENRICH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Number of resolved rows accumulated before flushing a batch back to the UI so
/// locations fill in progressively instead of all at once.
const ENRICH_FLUSH_BATCH: usize = 16;

pub async fn enrich_locations(
    db: xrat_db::Database,
    lookup: Arc<CachedLookup>,
    targets: Vec<(xrat_model::ConfigId, String)>,
    mut on_batch: impl FnMut(Vec<(xrat_model::ConfigId, xrat_support::geoip::EndpointGeoMeta)>),
) {
    let mut join_set = tokio::task::JoinSet::new();
    let mut pending = targets.into_iter();
    let spawn_next =
        |join_set: &mut tokio::task::JoinSet<_>,
         pending: &mut std::vec::IntoIter<(xrat_model::ConfigId, String)>| {
            if let Some((id, address)) = pending.next() {
                let lookup = lookup.clone();
                join_set.spawn(async move {
                    let host = xrat_support::geoip::address_host(&address);
                    let meta = tokio::time::timeout(
                        ENRICH_TIMEOUT,
                        xrat_support::geoip::enrich_address(&address, lookup.as_ref()),
                    )
                    .await
                    .unwrap_or_default();
                    (id, host, meta)
                });
            }
        };

    for _ in 0..ENRICH_CONCURRENCY {
        spawn_next(&mut join_set, &mut pending);
    }

    let mut batch = Vec::new();
    let mut persisted = std::collections::HashSet::new();
    while let Some(joined) = join_set.join_next().await {
        if let Ok((id, host, meta)) = joined {
            if let Some(host) = host
                && persisted.insert(host.clone())
            {
                persist_geo(&db, host, &meta).await;
            }
            if meta.has_lookup_metadata() {
                batch.push((id, meta));
                if batch.len() >= ENRICH_FLUSH_BATCH {
                    on_batch(std::mem::take(&mut batch));
                }
            }
        }
        spawn_next(&mut join_set, &mut pending);
    }

    if !batch.is_empty() {
        on_batch(batch);
    }
}

async fn persist_geo(
    db: &xrat_db::Database,
    host: String,
    meta: &xrat_support::geoip::EndpointGeoMeta,
) {
    let entry = xrat_db::GeoIpCacheUpsert {
        host,
        ip: None,
        country: meta.country.clone(),
        location: meta.location.clone(),
        asn: meta.asn.clone(),
        resolved_at: crate::app::ports::Clock::now_unix_secs(&crate::app::ports::SystemClock),
    };
    if let Err(error) = db.upsert_geoip_cache(&entry).await {
        tracing::debug!("geoip cache upsert failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use xrat_support::geoip::{GeoIpError, GeoIpLookup};
    #[derive(Debug)]
    struct FakeLookup;
    #[async_trait::async_trait]
    impl GeoIpLookup for FakeLookup {
        async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok((ip.to_string() != "9.9.9.9").then(|| "ZZ".into()))
        }
        async fn city(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(None)
        }
        async fn asn(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(None)
        }
        fn backend_name(&self) -> &'static str {
            "fake"
        }
    }
    #[tokio::test]
    async fn enrichment_batches_progress_and_persists_even_empty_geo_results() {
        let (context, _root) = crate::app::tests::TestAppBuilder::new("enrichment")
            .build_with_root()
            .await;
        let lookup = Arc::new(CachedLookup::new(
            Arc::new(FakeLookup),
            std::time::Duration::from_secs(60),
            32,
        ));
        let mut targets = (1..=17)
            .map(|id| (xrat_model::ConfigId(id), "8.8.8.8".to_string()))
            .collect::<Vec<_>>();
        targets.push((18.into(), "9.9.9.9".into()));
        let mut batches = Vec::new();
        enrich_locations(context.db.clone(), lookup, targets, |batch| {
            batches.push(batch)
        })
        .await;
        let updates = batches.iter().flatten().collect::<Vec<_>>();
        assert_eq!(
            updates
                .iter()
                .filter(|(_, meta)| meta.country.as_deref() == Some("ZZ"))
                .count(),
            17
        );
        assert_eq!(batches[0].len(), 16);
        let cache = context
            .db
            .get_fresh_geoip_cache(&["8.8.8.8".into(), "9.9.9.9".into()], 0)
            .await
            .unwrap();
        assert_eq!(cache.len(), 2);
        assert!(
            !cache
                .iter()
                .find(|row| row.host == "9.9.9.9")
                .unwrap()
                .has_location()
        );
    }
}
