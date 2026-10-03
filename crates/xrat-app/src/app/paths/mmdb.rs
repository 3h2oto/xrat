use std::path::{Path, PathBuf};

use crate::app::config::{self, AppConfig, defaults};
use crate::app::context::RuntimePaths;

pub fn resolve_mmdb_dir(runtime_paths: &RuntimePaths, app_config: &AppConfig) -> PathBuf {
    if app_config.mmdb.dir.is_absolute() {
        return app_config.mmdb.dir.clone();
    }

    runtime_paths.root_dir.join(&app_config.mmdb.dir)
}

pub fn mmdb_path_for(
    runtime_paths: &RuntimePaths,
    app_config: &AppConfig,
    configured_path: &Path,
    file_name: &str,
) -> PathBuf {
    if configured_path.is_absolute() {
        return configured_path.to_path_buf();
    }

    if configured_path == default_mmdb_relative_path(file_name) {
        return resolve_mmdb_dir(runtime_paths, app_config).join(file_name);
    }

    config::resolve_config_path(&runtime_paths.config_path, configured_path)
}

fn default_mmdb_relative_path(file_name: &str) -> PathBuf {
    PathBuf::from(defaults::DEFAULT_MMDB_DIR).join(file_name)
}

#[cfg(test)]
mod tests;
