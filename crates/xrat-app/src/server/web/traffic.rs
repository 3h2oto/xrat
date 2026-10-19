use super::security::WebState;
use crate::app::{
    context::AppContext,
    runtime_service::{RuntimeService, RuntimeStatusSnapshot},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use xrat_engines::xray::stats::{SingboxStatsSource, StatsSource, XrayStatsSource};
use xrat_web::types::{Traffic, TrafficSample};

const WINDOW_MS: f64 = 300_000.0;
const MAX_SAMPLES: usize = 102;

#[derive(Default)]
pub struct Collector {
    history: Mutex<History>,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Drop for Collector {
    fn drop(&mut self) {
        if let Ok(task) = self.task.get_mut()
            && let Some(task) = task.take()
        {
            task.abort();
        }
    }
}

impl Collector {
    pub fn snapshot(&self, session: Option<i64>, enabled: bool) -> Vec<TrafficSample> {
        let Ok(mut history) = self.history.lock() else {
            return Vec::new();
        };
        if !enabled || session.is_none() || session != history.session {
            return Vec::new();
        }
        history.prune(now_ms());
        history.samples.clone()
    }
}

#[derive(Default)]
struct History {
    session: Option<i64>,
    samples: Vec<TrafficSample>,
}

impl History {
    fn record(&mut self, session: Option<i64>, sample: TrafficSample) {
        if session != self.session
            || self
                .samples
                .last()
                .is_some_and(|last| last.sampled_at_ms >= sample.sampled_at_ms)
        {
            self.samples.clear();
        }
        self.session = session;
        if session.is_none() {
            self.samples.clear();
            return;
        }
        let at = sample.sampled_at_ms;
        self.samples.push(sample);
        self.prune(at);
    }

    fn prune(&mut self, at: f64) {
        self.samples
            .retain(|sample| (at - WINDOW_MS..=at).contains(&sample.sampled_at_ms));
        if self.samples.len() > MAX_SAMPLES {
            self.samples.drain(..self.samples.len() - MAX_SAMPLES);
        }
    }
}

pub fn start(web: &Arc<WebState>) {
    if web.context.is_none() {
        return;
    }
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        return;
    };
    let Ok(mut task) = web.traffic.task.lock() else {
        return;
    };
    if task.is_some() {
        return;
    }
    let weak = Arc::downgrade(web);
    *task = Some(runtime.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let Some(web) = weak.upgrade() else { break };
            let result = tokio::time::timeout(
                Duration::from_secs(2),
                collect(web.context.as_ref().unwrap()),
            )
            .await;
            if let Ok(mut history) = web.traffic.history.lock() {
                let (session, traffic) = result
                    .ok()
                    .and_then(Result::ok)
                    .unwrap_or((history.session, None));
                history.record(
                    session,
                    TrafficSample {
                        sampled_at_ms: now_ms(),
                        traffic,
                    },
                );
            }
        }
    }));
}

async fn collect(context: &AppContext) -> crate::app::Result<(Option<i64>, Option<Traffic>)> {
    let mut current = context.clone();
    current.app_config = crate::app::config::load(&context.runtime_paths.config_path)?;
    if !current.app_config.runtime.stats.enabled {
        return Ok((None, None));
    }
    let snapshot = RuntimeService::new(&current).status().await?;
    let session = snapshot
        .session
        .as_ref()
        .filter(|_| snapshot.pid_running)
        .map(|session| session.id);
    let traffic = sample(&current, &snapshot).await;
    let latest = current
        .db
        .get_latest_runtime_session()
        .await?
        .map(|session| session.id);
    if latest != session {
        return Ok((None, None));
    }
    Ok((session, traffic))
}

pub async fn sample(context: &AppContext, snapshot: &RuntimeStatusSnapshot) -> Option<Traffic> {
    if !snapshot.pid_running || !context.app_config.runtime.stats.enabled {
        return None;
    }
    let settings = &context.app_config.runtime.stats;
    let source: Box<dyn StatsSource> = if context.app_config.runtime.engine == "sing-box"
        || snapshot
            .active_config
            .as_ref()
            .is_some_and(|config| config.protocol == "hysteria2")
    {
        Box::new(SingboxStatsSource::new(
            &format!("{}:{}", settings.host, settings.port),
            None,
        ))
    } else {
        Box::new(XrayStatsSource::new(&settings.host, settings.port))
    };
    tokio::time::timeout(Duration::from_millis(500), source.sample())
        .await
        .ok()
        .and_then(Result::ok)
        .map(|sample| Traffic {
            uploaded_bytes: sample.uplink_total,
            downloaded_bytes: sample.downlink_total,
        })
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}

#[cfg(test)]
#[path = "traffic_tests.rs"]
mod tests;
