use super::*;

#[test]
fn resolves_binary_paths_from_config_file() {
    let root_dir = temp_root("xrat-runtime-config-binaries");
    let config_path = root_dir.join("config.toml");
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    // An absolute configured path is used as is, while a relative one is
    // resolved against the directory that holds the config file.
    let absolute_v2ray = temp_root("xrat-runtime-absolute-v2ray").join("v2ray");
    std::fs::write(
        &config_path,
        format!(
            "[paths]\nxray = \"bin/xray\"\nv2ray = {:?}\nsing_box = \"bin/sing-box\"\n",
            absolute_v2ray.to_string_lossy()
        ),
    )
    .expect("config should be written");

    let cli = cli_for_config(&config_path);
    let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

    assert_eq!(runtime_paths.xray_path, root_dir.join("bin/xray"));
    assert_eq!(runtime_paths.v2ray_path, absolute_v2ray);
    assert_eq!(runtime_paths.sing_box_path, root_dir.join("bin/sing-box"));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
}

#[test]
fn cli_binary_paths_override_config_file() {
    let root_dir = temp_root("xrat-runtime-cli-binaries");
    let config_path = root_dir.join("config.toml");
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    std::fs::write(
        &config_path,
        "[paths]\nxray = \"bin/xray\"\nv2ray = \"bin/v2ray\"\nsing_box = \"bin/sing-box\"\n",
    )
    .expect("config should be written");

    let cli = Cli::parse_from([
        "xrat",
        "--config",
        config_path.to_str().unwrap(),
        "--xray",
        "/custom/xray",
        "--v2ray",
        "/custom/v2ray",
        "--sing-box",
        "/custom/sing-box",
        "list",
        "configs",
    ]);
    let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

    assert_eq!(runtime_paths.xray_path, PathBuf::from("/custom/xray"));
    assert_eq!(runtime_paths.v2ray_path, PathBuf::from("/custom/v2ray"));
    assert_eq!(
        runtime_paths.sing_box_path,
        PathBuf::from("/custom/sing-box")
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
}
