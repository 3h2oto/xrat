use xrat_web::types::{ConfigRow, ConfigTest, DashboardData, LatencyMetric, Traffic};

pub const METRICS: [LatencyMetric; 3] = [
    LatencyMetric::Tcp,
    LatencyMetric::Icmp,
    LatencyMetric::RealDelay,
];

pub fn label(metric: LatencyMetric) -> &'static str {
    match metric {
        LatencyMetric::Tcp => "TCP",
        LatencyMetric::Icmp => "ICMP",
        LatencyMetric::RealDelay => "Real delay",
    }
}
pub fn key(metric: LatencyMetric) -> &'static str {
    match metric {
        LatencyMetric::Tcp => "tcp",
        LatencyMetric::Icmp => "icmp",
        LatencyMetric::RealDelay => "real_delay",
    }
}
pub fn measurement(test: &ConfigTest, metric: LatencyMetric) -> (Option<bool>, Option<i64>) {
    match metric {
        LatencyMetric::Tcp => (test.tcp_ok, test.tcp_ms),
        LatencyMetric::Icmp => (test.icmp_ok, test.icmp_ms),
        LatencyMetric::RealDelay => (test.real_delay_ok, test.real_delay_ms),
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Health {
    pub passed: usize,
    pub failed: usize,
    pub untested: usize,
    pub average: Option<f64>,
    pub samples: usize,
}
pub fn health(configs: &[ConfigRow], metric: LatencyMetric) -> Health {
    let mut result = Health::default();
    let mut total = 0.0;
    for config in configs.iter().filter(|row| row.enabled && !row.deleted) {
        let (ok, value) = config
            .test
            .as_ref()
            .map(|test| measurement(test, metric))
            .unwrap_or_default();
        match ok {
            Some(true) => result.passed += 1,
            Some(false) => result.failed += 1,
            None => result.untested += 1,
        }
        if ok == Some(true)
            && let Some(value) = value.filter(|value| *value >= 0)
        {
            total += value as f64;
            result.samples += 1;
        }
    }
    result.average = (result.samples > 0).then(|| total / result.samples as f64);
    result
}

pub fn config_distribution(configs: &[ConfigRow]) -> [usize; 5] {
    let mut counts = [0; 5];
    for config in configs.iter().filter(|row| !row.deleted) {
        let results = config
            .test
            .as_ref()
            .map(|test| [test.real_delay_ok, test.tcp_ok, test.icmp_ok])
            .unwrap_or_default();
        let index = results
            .iter()
            .position(|ok| *ok == Some(true))
            .unwrap_or_else(|| {
                if results.iter().any(Option::is_some) {
                    3
                } else {
                    4
                }
            });
        counts[index] += 1;
    }
    counts
}

#[derive(Clone, Debug, PartialEq)]
pub struct RatePoint {
    pub at: f64,
    pub down: Option<f64>,
    pub up: Option<f64>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TrafficHistory {
    pub points: Vec<RatePoint>,
    session: Option<i64>,
    baseline: Option<(f64, u64, u64)>,
}
impl TrafficHistory {
    pub fn interrupt(&mut self) {
        self.baseline = None;
    }
    pub fn restore(&mut self, data: &DashboardData, running: bool) {
        *self = Self::default();
        for sample in &data.traffic_history {
            self.record_at(data, sample.traffic.as_ref(), running, sample.sampled_at_ms);
        }
    }
    #[cfg(test)]
    pub fn record(&mut self, data: &DashboardData, traffic: Option<&Traffic>, running: bool) {
        self.record_at(data, traffic, running, data.sampled_at_ms);
    }
    fn record_at(
        &mut self,
        data: &DashboardData,
        traffic: Option<&Traffic>,
        running: bool,
        at: f64,
    ) {
        if data.session_id != self.session || !running || !data.stats_enabled {
            self.points.clear();
            self.baseline = None;
            self.session = data.session_id;
        }
        if !running || !data.stats_enabled || data.session_id.is_none() || !at.is_finite() {
            return;
        }
        if self.points.last().is_some_and(|point| at == point.at) {
            return;
        }
        if self.points.last().is_some_and(|point| at < point.at) {
            self.points.clear();
            self.baseline = None;
        }
        let mut point = RatePoint {
            at,
            down: None,
            up: None,
        };
        if let Some(traffic) = traffic {
            if let Some((before, down, up)) = self.baseline {
                let elapsed = (at - before) / 1000.0;
                if traffic.downloaded_bytes < down || traffic.uploaded_bytes < up {
                    self.points.clear();
                    self.baseline = None;
                    self.points.push(point);
                    return;
                } else if (0.25..=15.0).contains(&elapsed) {
                    point.down = Some((traffic.downloaded_bytes - down) as f64 / elapsed);
                    point.up = Some((traffic.uploaded_bytes - up) as f64 / elapsed);
                }
            }
            self.baseline = Some((at, traffic.downloaded_bytes, traffic.uploaded_bytes));
        } else {
            self.baseline = None;
        }
        self.points.push(point);
        self.points.retain(|point| point.at >= at - 300_000.0);
        if self.points.len() > 120 {
            self.points.drain(..self.points.len() - 120);
        }
    }
    pub fn latest(&self) -> Option<&RatePoint> {
        self.points.last()
    }
}

pub fn bytes(value: f64) -> String {
    if !value.is_finite() || value < 0.0 {
        return "—".into();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut scaled = value;
    let mut index = 0;
    while scaled >= 1024.0 && index < units.len() - 1 {
        scaled /= 1024.0;
        index += 1;
    }
    if index == 0 {
        format!("{scaled:.0} {}", units[index])
    } else {
        format!("{scaled:.1} {}", units[index])
    }
}
pub fn milliseconds(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.1} ms"))
        .unwrap_or_else(|| "—".into())
}
