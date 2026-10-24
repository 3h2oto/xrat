mod process;
pub use process::ProcessRuntimeEngineProbe;

use crate::app::context::AppContext;
use crate::app::ports::{EngineInfo, RuntimeEngineProbe};

pub async fn probe_engines(
    context: &AppContext,
    probe: &dyn RuntimeEngineProbe,
) -> Vec<EngineInfo> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        use super::diagnostics::{DiagnosticsPayload, DiagnosticsRequest};
        let rows = match client.diagnostics(DiagnosticsRequest::Engines {}).await {
            Ok(DiagnosticsPayload::Engines(rows)) => rows,
            result => {
                tracing::debug!(?result, "machine engine metadata unavailable");
                Vec::new()
            }
        };
        return ["xray", "sing-box"]
            .into_iter()
            .map(|name| {
                rows.iter()
                    .find(|row| row.name == name)
                    .map(|row| EngineInfo {
                        name,
                        available: row.available,
                        version: row.version.clone(),
                    })
                    .unwrap_or(EngineInfo {
                        name,
                        available: false,
                        version: None,
                    })
            })
            .collect();
    }
    probe_local_engines(context, probe).await
}

pub(crate) async fn probe_local_engines(
    context: &AppContext,
    probe: &dyn RuntimeEngineProbe,
) -> Vec<EngineInfo> {
    let (xray, sing_box) = tokio::join!(
        probe.probe("xray", &context.runtime_paths.xray_path),
        probe.probe("sing-box", &context.runtime_paths.sing_box_path)
    );
    [("xray", xray), ("sing-box", sing_box)].into_iter().map(|(name, result)| {
        result.unwrap_or_else(|error| {
            tracing::debug!(operation = "engine_version_probe", engine = name, %error, "optional engine probe failed");
            EngineInfo { name, available: false, version: None }
        })
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    struct FakeProbe;
    #[async_trait::async_trait]
    impl RuntimeEngineProbe for FakeProbe {
        async fn probe(&self, name: &'static str, _path: &Path) -> crate::app::Result<EngineInfo> {
            if name == "sing-box" {
                return Err(crate::app::AppError::InvalidArgument(
                    "missing fake engine".into(),
                ));
            }
            Ok(EngineInfo {
                name,
                available: true,
                version: Some("26.1.0".into()),
            })
        }
    }
    #[tokio::test]
    async fn engine_failures_are_isolated_using_an_injected_probe() {
        let (context, _root) = crate::app::tests::TestAppBuilder::new("fake-engine")
            .build_with_root()
            .await;
        let engines = probe_engines(&context, &FakeProbe).await;
        assert_eq!(
            engines,
            vec![
                EngineInfo {
                    name: "xray",
                    available: true,
                    version: Some("26.1.0".into())
                },
                EngineInfo {
                    name: "sing-box",
                    available: false,
                    version: None
                },
            ]
        );
    }
}
