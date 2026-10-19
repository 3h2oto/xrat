#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn step_script(name: &str) -> String {
    let workflow = include_str!("../.github/workflows/release.yml");
    workflow
        .split_once(&format!("- name: {name}\n"))
        .unwrap()
        .1
        .split_once("        run: |\n")
        .unwrap()
        .1
        .lines()
        .take_while(|line| line.is_empty() || line.starts_with("          "))
        .map(|line| line.strip_prefix("          ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn executable(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir(root.path().join("bin")).unwrap();
    fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = \"0.31.0\"\n",
    )
    .unwrap();
    fs::write(repo.join(".gitignore"), "/crates/xrat-app/web-assets/\n").unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Release test",
            "-c",
            "user.email=release@example.com",
            "commit",
            "-qm",
            "fixture",
        ],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
    }
    root
}

fn run_step(root: &Path, name: &str) -> std::process::Output {
    Command::new("bash")
        .args(["-c", &step_script(name)])
        .current_dir(root.join("repo"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("PUBLISH_CALLS", root.join("calls"))
        .env("SOURCE_RUN_FILE", root.join("source.json"))
        .env("RUNNER_TEMP", root)
        .env("RELEASE_TAG", "v0.31.0")
        .env("ASSETS_RUN_ID", "123")
        .env("GITHUB_REPOSITORY", "example/xrat")
        .output()
        .unwrap()
}

#[test]
fn partial_publication_allows_generated_assets_only_for_app_and_rejects_dirty_source() {
    let root = fixture();
    let repo = root.path().join("repo");
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/bash\nprintf '%s\\n' \"$*\" >> \"$PUBLISH_CALLS\"\n",
    );
    executable(
        &root.path().join("bin/curl"),
        "#!/bin/bash\ncase \"$*\" in */xrat-app/0.31.0|*/xrat-sdk/0.31.0|*/xrat/0.31.0) exit 22;; *) exit 0;; esac\n",
    );
    let assets = repo.join("crates/xrat-app/web-assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("xrat_web_bg.wasm"), "generated").unwrap();
    let name = "Publish workspace crates in dependency order";
    let output = run_step(root.path(), name);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("calls")).unwrap(),
        "publish -p xrat-app --locked --allow-dirty\npublish -p xrat-sdk --locked\npublish -p xrat --locked\n"
    );
    fs::remove_file(root.path().join("calls")).unwrap();
    let manifest = fs::read_to_string(repo.join("Cargo.toml")).unwrap();
    fs::write(repo.join("Cargo.toml"), format!("{manifest}# dirty\n")).unwrap();
    assert!(!run_step(root.path(), name).status.success());
    assert!(!root.path().join("calls").exists());
    fs::write(repo.join("Cargo.toml"), manifest).unwrap();
    fs::write(repo.join("unexpected.txt"), "untracked").unwrap();
    assert!(!run_step(root.path(), name).status.success());
    assert!(!root.path().join("calls").exists());
}

#[test]
fn recovery_requires_assets_from_the_same_release_commit_tag_and_workflow() {
    let root = fixture();
    executable(
        &root.path().join("bin/gh"),
        "#!/bin/bash\ncat \"$SOURCE_RUN_FILE\"\n",
    );
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root.path().join("repo"))
        .output()
        .unwrap();
    let sha = String::from_utf8(sha.stdout).unwrap();
    let source = format!(
        "{{\"head_sha\":\"{}\",\"head_branch\":\"v0.31.0\",\"event\":\"push\",\"path\":\".github/workflows/release.yml\"}}",
        sha.trim()
    );
    let name = "Verify release source and asset provenance";
    fs::write(root.path().join("source.json"), &source).unwrap();
    let output = run_step(root.path(), name);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for invalid in [
        source.replace(sha.trim(), "wrong-commit"),
        source.replace("v0.31.0", "v0.30.3"),
        source.replace("release.yml", "other.yml"),
        source.replace("push", "pull_request"),
    ] {
        fs::write(root.path().join("source.json"), invalid).unwrap();
        assert!(!run_step(root.path(), name).status.success());
    }
}
