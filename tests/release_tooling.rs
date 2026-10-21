use std::{fs, path::Path, process::Command};
use toml_edit::DocumentMut;

const INSTALL_SCRIPT: &str = include_str!("../install.ps1");

fn versions_match(content: &str) -> bool {
    let manifest: DocumentMut = content.parse().unwrap();
    let current = manifest["workspace"]["package"]["version"]
        .as_str()
        .unwrap();
    manifest["workspace"]["dependencies"]
        .as_table_like()
        .unwrap()
        .iter()
        .all(|(_, dependency)| {
            dependency.get("path").is_none()
                || dependency.get("version").and_then(|value| value.as_str()) == Some(current)
        })
}
#[test]
fn workspace_versions_match() {
    assert!(
        versions_match(include_str!("../Cargo.toml")),
        "workspace internal dependency versions differ"
    );
}
#[test]
fn detects_unsynchronized_versions() {
    assert!(!versions_match(
        "[workspace.package]\nversion=\"0.21.0\"\n[workspace.dependencies]\nxrat-model={path=\"model\",version=\"0.20.0\"}"
    ));
}
#[test]
fn version_recipe_updates_versions_and_preserves_unrelated_content() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("src")).unwrap();
    fs::write(root.path().join("src/lib.rs"), "").unwrap();
    fs::create_dir_all(root.path().join("model/src")).unwrap();
    fs::write(root.path().join("model/src/lib.rs"), "").unwrap();
    fs::write(
        root.path().join("model/Cargo.toml"),
        "[package]\nname=\"xrat-model\"\nversion.workspace=true\nedition.workspace=true\n",
    )
    .unwrap();
    let manifest = root.path().join("Cargo.toml");
    fs::write(&manifest, "# retained\n[package]\nname=\"fixture\"\nversion.workspace=true\nedition.workspace=true\n[workspace]\nmembers=[\"model\"]\n[workspace.package]\nversion = \"0.20.0\"\nedition=\"2024\"\n[workspace.dependencies]\nxrat-model = { path = \"model\", version = \"0.20.0\" }\nserde = \"1.0\"\n").unwrap();
    let recipe = Path::new(env!("CARGO_MANIFEST_DIR")).join("Justfile");
    let run = |version| {
        Command::new("just")
            .env("XDG_RUNTIME_DIR", root.path())
            .arg("--justfile")
            .arg(&recipe)
            .arg("--working-directory")
            .arg(root.path())
            .args(["set-version", version])
            .output()
            .unwrap()
    };
    let result = run("0.21.0");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let content = fs::read_to_string(&manifest).unwrap();
    assert!(content.starts_with("# retained"));
    assert!(content.contains("serde = \"1.0\""));
    assert_eq!(content.matches("0.21.0").count(), 2);
    assert!(versions_match(&content));
    assert!(!run("invalid").status.success());
    assert_eq!(fs::read_to_string(manifest).unwrap(), content);
}
#[test]
fn install_script_exists() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.ps1");
    assert!(
        script.is_file(),
        "install.ps1 is missing from the repository root"
    );
}
#[test]
fn install_script_declares_windows_targets() {
    assert!(
        INSTALL_SCRIPT.contains("x86_64-pc-windows-msvc"),
        "install.ps1 must install the x86_64 Windows artifact"
    );
    assert!(
        INSTALL_SCRIPT.contains("aarch64-pc-windows-msvc"),
        "install.ps1 must install the ARM64 Windows artifact"
    );
}
#[test]
fn install_script_verifies_checksums() {
    assert!(
        INSTALL_SCRIPT.contains("SHASUMS256.txt"),
        "install.ps1 must verify downloads against the release checksums"
    );
    assert!(
        INSTALL_SCRIPT.contains("Get-FileHash"),
        "install.ps1 must hash the downloaded archive"
    );
    assert!(
        INSTALL_SCRIPT.contains("SHA256"),
        "install.ps1 must use SHA-256 checksums"
    );
}
#[test]
fn install_script_rejects_32bit_windows() {
    assert!(
        INSTALL_SCRIPT.contains("PROCESSOR_ARCHITEW6432"),
        "install.ps1 must read the WOW64 architecture variable"
    );
    assert!(
        INSTALL_SCRIPT.contains("PROCESSOR_ARCHITECTURE"),
        "install.ps1 must read the processor architecture variable"
    );
}
#[test]
fn install_script_downloads_the_zip_asset() {
    assert!(
        INSTALL_SCRIPT.contains("releases/download"),
        "install.ps1 must download from the GitHub release assets"
    );
    assert!(
        INSTALL_SCRIPT.contains("xrat-${tag}-${target}.zip"),
        "install.ps1 must build the release archive name from tag and target"
    );
}
