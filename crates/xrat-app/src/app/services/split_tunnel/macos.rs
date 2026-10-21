use super::{AppSource, DiscoveredApp};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn bundle_directory(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .filter(|parent| parent.extension().is_some_and(|ext| ext == "app"))
        .last()
        .map(Path::to_path_buf)
}

pub(super) fn discover_bundles(dirs: &[PathBuf]) -> Vec<DiscoveredApp> {
    let mut pending = dirs.to_vec();
    let mut apps = BTreeMap::new();
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                if path.extension().is_some_and(|ext| ext == "app") {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let rule_entry = format!("{}/", path.display());
                    apps.insert(
                        rule_entry.clone(),
                        DiscoveredApp {
                            name: name.clone(),
                            desktop_id: None,
                            process_name: name,
                            exec_path: Some(path.display().to_string()),
                            rule_entry,
                            source: AppSource::Desktop,
                        },
                    );
                } else {
                    pending.push(path);
                }
            }
        }
    }
    apps.into_values().collect()
}

#[cfg(target_os = "macos")]
pub(super) fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }
    dirs
}

#[cfg(target_os = "macos")]
pub(super) fn running_apps() -> Vec<DiscoveredApp> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        false,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );
    let mut apps = BTreeMap::new();
    for process in system.processes().values() {
        let Some(path) = process.exe() else {
            continue;
        };
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if matches!(name, "xrat" | "xray" | "sing-box" | "v2ray") {
            continue;
        }
        let bundle = bundle_directory(path);
        let rule_entry = bundle.as_ref().map_or_else(
            || path.display().to_string(),
            |bundle| format!("{}/", bundle.display()),
        );
        apps.entry(rule_entry.clone())
            .or_insert_with(|| DiscoveredApp {
                name: bundle
                    .as_ref()
                    .and_then(|bundle| bundle.file_stem())
                    .unwrap_or_else(|| std::ffi::OsStr::new(name))
                    .to_string_lossy()
                    .to_string(),
                desktop_id: None,
                process_name: name.to_string(),
                exec_path: Some(path.display().to_string()),
                rule_entry,
                source: AppSource::Running,
            });
    }
    apps.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_nested_bundles_and_keeps_all_helpers_in_the_rule() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("Utilities").join("Demo App.app");
        std::fs::create_dir_all(bundle.join("Contents").join("MacOS")).unwrap();
        std::fs::create_dir_all(
            bundle
                .join("Contents")
                .join("Frameworks")
                .join("Worker.app"),
        )
        .unwrap();
        let apps = discover_bundles(&[temp.path().to_path_buf()]);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Demo App");
        assert_eq!(apps[0].rule_entry, format!("{}/", bundle.display()));
        assert_eq!(
            bundle_directory(
                &bundle
                    .join("Contents")
                    .join("Frameworks")
                    .join("Worker.app")
                    .join("Contents")
                    .join("MacOS")
                    .join("worker")
            ),
            Some(bundle.clone())
        );
        assert_eq!(
            bundle_directory(&bundle.join("Contents").join("MacOS").join("demo")),
            Some(bundle)
        );
    }
}
