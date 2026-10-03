use crate::app::ports::{EngineInfo, RuntimeEngineProbe};
use std::path::Path;

pub struct ProcessRuntimeEngineProbe;

#[async_trait::async_trait]
impl RuntimeEngineProbe for ProcessRuntimeEngineProbe {
    async fn probe(&self, name: &'static str, path: &Path) -> crate::app::Result<EngineInfo> {
        probe_with_spawner(
            name,
            path,
            std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
        )
        .await
    }
}

pub async fn probe_with_spawner(
    name: &'static str,
    path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> crate::app::Result<EngineInfo> {
    let mut command = xrat_support::process::Command::with_spawner(path, spawner);
    command.arg("version").kill_on_drop(true);
    let output = tokio::time::timeout(std::time::Duration::from_secs(2), command.output_async())
        .await
        .map_err(|_| {
            crate::app::AppError::InvalidArgument(format!("{name} version probe timed out"))
        })??;
    if !output.status.success() {
        return Err(crate::app::AppError::InvalidArgument(format!(
            "{name} version probe failed: {}",
            output.status
        )));
    }
    Ok(EngineInfo {
        name,
        available: true,
        version: parse_engine_version(&String::from_utf8_lossy(&output.stdout)),
    })
}

fn parse_engine_version(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|token| token.trim_start_matches('v'))
        .find(|token| {
            token.contains('.') && token.chars().next().is_some_and(|c| c.is_ascii_digit())
        })
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_engine_banners() {
        assert_eq!(
            parse_engine_version("Xray 26.1.0 (Xray, Penetrates Everything.)"),
            Some("26.1.0".into())
        );
        assert_eq!(
            parse_engine_version("sing-box version v1.13.21"),
            Some("1.13.21".into())
        );
        assert_eq!(parse_engine_version("unrecognized banner"), None);
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn hung_engine_version_probe_is_bounded() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let script = root.path().join("hung-engine");
        std::fs::write(&script, "#!/bin/sh\nexec sleep 30\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(4),
            ProcessRuntimeEngineProbe.probe("xray", &script),
        )
        .await
        .unwrap();
        assert!(
            matches!(result, Err(crate::app::AppError::InvalidArgument(message)) if message.contains("timed out"))
        );
    }
}
