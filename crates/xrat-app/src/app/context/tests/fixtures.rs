use super::*;

pub(super) fn temp_root(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ))
}

pub(super) fn cli_for_config(config_path: &std::path::Path) -> Cli {
    Cli::parse_from([
        "xrat",
        "--config",
        config_path.to_str().unwrap(),
        "list",
        "configs",
    ])
}
