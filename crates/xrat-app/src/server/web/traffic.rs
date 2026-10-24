use super::security::WebState;
use crate::app::context::AppContext;
pub(super) use crate::app::services::runtime_traffic::sample;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
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
    let snapshot = crate::app::services::runtime_traffic::collect(context).await?;
    Ok((snapshot.session_id, snapshot.traffic))
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
