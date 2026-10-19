use super::ResolvedTestSettings;
use xrat_model::ConfigId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestMetric {
    Icmp,
    Tcp,
    RealDelay,
    Download,
    Upload,
}

#[derive(Debug, Clone)]
pub(crate) enum TestActivity {
    Queued {
        config_id: ConfigId,
        metrics: Vec<TestMetric>,
    },
    Started {
        config_id: ConfigId,
        metric: TestMetric,
    },
    Measured {
        config_id: ConfigId,
        metric: TestMetric,
        passed: bool,
    },
    Finished {
        config_id: ConfigId,
        failed: bool,
    },
}

pub(crate) type TestActivitySender = tokio::sync::mpsc::UnboundedSender<TestActivity>;

pub(crate) fn queued(settings: &ResolvedTestSettings, config_id: ConfigId) {
    let Some(sender) = &settings.activity else {
        return;
    };
    let metrics = [
        (TestMetric::Icmp, settings.run_icmp),
        (TestMetric::Tcp, settings.run_tcp),
        (TestMetric::RealDelay, settings.run_real_delay),
        (TestMetric::Download, settings.run_download),
        (
            TestMetric::Upload,
            settings.run_download && settings.run_upload,
        ),
    ]
    .into_iter()
    .filter_map(|(metric, enabled)| enabled.then_some(metric))
    .collect();
    let _ = sender.send(TestActivity::Queued { config_id, metrics });
}

pub(crate) fn started(settings: &ResolvedTestSettings, config_id: ConfigId, metric: TestMetric) {
    if let Some(sender) = &settings.activity {
        let _ = sender.send(TestActivity::Started { config_id, metric });
    }
}

pub(crate) fn measured(
    settings: &ResolvedTestSettings,
    config_id: ConfigId,
    metric: TestMetric,
    passed: bool,
) {
    if let Some(sender) = &settings.activity {
        let _ = sender.send(TestActivity::Measured {
            config_id,
            metric,
            passed,
        });
    }
}

#[cfg(test)]
mod tests;
