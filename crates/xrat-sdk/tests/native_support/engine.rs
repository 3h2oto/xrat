use std::{os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
use xrat_sdk::prober::ProbeEngineKind;

pub struct Engine {
    pub kind: ProbeEngineKind,
    pub binary: PathBuf,
    pub pid_file: PathBuf,
    directory: tempfile::TempDir,
}

impl Engine {
    pub fn new(kind: ProbeEngineKind) -> Self {
        let variable = match kind {
            ProbeEngineKind::Xray => "XRAT_SDK_XRAY_BINARY",
            ProbeEngineKind::Singbox => "XRAT_SDK_SINGBOX_BINARY",
        };
        let binary = std::env::var(variable).unwrap_or_else(|_| panic!("set {variable}"));
        let directory = tempfile::tempdir().unwrap();
        let pid_file = directory.path().join("pid");
        let wrapper = directory.path().join("engine");
        let quoted = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
        std::fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\nprintf '%s' \"$$\" > {}\nexec {} \"$@\"\n",
                quoted(pid_file.to_str().unwrap()),
                quoted(&binary)
            ),
        )
        .unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            kind,
            binary: wrapper,
            pid_file,
            directory,
        }
    }

    pub fn replace_command(&self, command: &str) {
        std::fs::write(
            &self.binary,
            format!(
                "#!/bin/sh\nprintf '%s' \"$$\" > '{}'\nexec {command}\n",
                self.pid_file.display()
            ),
        )
        .unwrap();
    }

    pub async fn assert_stopped(&self) {
        let pid = std::fs::read_to_string(&self.pid_file).unwrap();
        let path = PathBuf::from(format!("/proc/{pid}"));
        for _ in 0..100 {
            if !path.exists() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!(
            "engine {pid} survived probe completion/cancellation in {}",
            self.directory.path().display()
        );
    }
}
