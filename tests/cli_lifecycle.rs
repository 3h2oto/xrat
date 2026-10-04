use std::path::Path;
use std::process::{Command, Output};

fn run(home: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_xrat"))
        .env("XRAT_PATH", home)
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .expect("xrat binary should start");
    assert!(
        output.status.success(),
        "xrat {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn cli_config_lifecycle_uses_shared_services() {
    let root = tempfile::tempdir().expect("temporary home should exist");
    let home = root.path().join("home");
    let fixture = root.path().join("nodes.txt");
    std::fs::write(
        &fixture,
        "vless://11111111-1111-1111-1111-111111111111@192.0.2.1:443?security=none&type=tcp#smoke-node\n",
    )
    .expect("fixture should write");

    run(&home, &["init"]);
    run(
        &home,
        &["import", fixture.to_str().expect("UTF-8 fixture path")],
    );

    let table = run(&home, &["list", "configs"]);
    assert!(String::from_utf8_lossy(&table.stdout).contains("smoke-node"));

    let json = run(&home, &["list", "configs", "--format", "json"]);
    let configs: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let config = &configs.as_array().unwrap()[0];
    assert_eq!(config["name"], "smoke-node");
    let reference = config["ref"].as_str().unwrap();

    let shown = run(&home, &["show", "config", reference, "--json"]);
    let detail: serde_json::Value = serde_json::from_slice(&shown.stdout).unwrap();
    assert_eq!(detail["ref"], reference);

    run(&home, &["delete", "config", reference]);
    let active = run(&home, &["list", "configs", "--format", "json"]);
    assert!(String::from_utf8_lossy(&active.stdout).contains("No configs matched."));
    let deleted = run(&home, &["list", "configs", "--deleted", "--format", "json"]);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&deleted.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );

    run(&home, &["purge", "--yes"]);
    let all = run(&home, &["list", "configs", "--all", "--format", "json"]);
    assert!(String::from_utf8_lossy(&all.stdout).contains("No configs matched."));
}
