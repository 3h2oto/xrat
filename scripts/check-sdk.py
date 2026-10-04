#!/usr/bin/env python3
"""Verify the standalone SDK consumer and its optional application boundary."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def run(args: list[str], *, cwd: Path = ROOT, env: dict[str, str] | None = None) -> None:
    subprocess.run(args, cwd=cwd, env=env, check=True)


def check_dependencies(cwd: Path) -> None:
    tree = subprocess.check_output(
        ["cargo", "tree", "--locked", "-p", "xrat-sdk", "--edges", "normal", "--prefix", "none", "--format", "{p}"],
        cwd=cwd, text=True)
    names = {line.split()[0] for line in tree.splitlines()}
    forbidden = {"xrat-app", "xrat-db", "clap", "ratatui", "crossterm", "sqlx", "arboard", "axum", "tonic", "prost"}
    assert not names & forbidden, f"Default SDK includes application dependencies: {names & forbidden}"


def consumer(registry_version: str | None) -> None:
    with tempfile.TemporaryDirectory(prefix="xrat-sdk-consumer-") as temporary:
        destination = Path(temporary)
        shutil.copytree(ROOT / "testdata/sdk-consumer", destination, dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns("Cargo.lock", "target"))
        manifest = destination / "Cargo.toml"
        content = manifest.read_text()
        dependency = 'xrat-sdk = { path = "../../crates/xrat-sdk" }'
        if registry_version:
            manifest.write_text(content.replace(dependency + "\n", ""))
            for attempt in range(5):
                result = subprocess.run(["cargo", "add", "xrat-sdk"], cwd=destination)
                if result.returncode == 0:
                    break
                if attempt == 4:
                    result.check_returncode()
                time.sleep(20)
        else:
            manifest.write_text(content.replace(dependency, f"xrat-sdk = {{ path = {json.dumps(str(ROOT / 'crates/xrat-sdk'))} }}"))
            shutil.copyfile(ROOT / "Cargo.lock", destination / "Cargo.lock")
        environment = os.environ.copy()
        environment["CARGO_TARGET_DIR"] = str(ROOT / "target")
        run(["cargo", "run", "--manifest-path", str(manifest)], cwd=destination, env=environment)
        check_dependencies(destination)
        metadata = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--format-version", "1"], cwd=destination, text=True))
        sdk = next(package for package in metadata["packages"] if package["name"] == "xrat-sdk")
        if registry_version:
            assert sdk["version"] == registry_version, sdk
            assert sdk["source"] and sdk["source"].startswith("registry+"), sdk
            assert all(package["source"] for package in metadata["packages"]
                       if package["name"].startswith("xrat-")), "internal path dependency in registry consumer"
        print(f"Standalone consumer verified: xrat-sdk {sdk['version']}", flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", metavar="VERSION", help="verify only the published registry consumer")
    args = parser.parse_args()
    if args.registry:
        consumer(args.registry)
        return
    check_dependencies(ROOT)
    run(["cargo", "test", "--locked", "-p", "xrat-sdk"])
    run(["cargo", "test", "--locked", "-p", "xrat-sdk", "--all-targets", "--features", "services"])
    run(["cargo", "clippy", "--locked", "-p", "xrat-sdk", "--all-targets", "--all-features", "--", "-D", "warnings"])
    environment = os.environ.copy()
    environment["RUSTDOCFLAGS"] = "-D warnings"
    run(["cargo", "doc", "--locked", "-p", "xrat-sdk", "--all-features", "--no-deps"], env=environment)
    consumer(None)


if __name__ == "__main__":
    main()
