#!/usr/bin/env python3
"""Update or check the workspace's package and internal dependency versions."""

import argparse
from pathlib import Path
import re
import tomllib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", nargs="?")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check == bool(args.version):
        parser.error("provide a version or --check")
    manifest = Path(__file__).resolve().parents[1] / "Cargo.toml"
    content = manifest.read_text()
    data = tomllib.loads(content)
    current = data["workspace"]["package"]["version"]
    dependencies = data["workspace"]["dependencies"]
    if args.check:
        mismatched = [
            name for name, dependency in dependencies.items()
            if "path" in dependency and dependency.get("version") != current
        ]
        if mismatched:
            parser.exit(1, f"internal versions differ from {current}: {', '.join(mismatched)}\n")
        return
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", args.version):
        parser.error("version must use MAJOR.MINOR.PATCH with optional prerelease/build suffix")
    content = re.sub(
        r'(\[workspace.package\]\s*\nversion\s*=\s*")[^"]+',
        lambda match: match[1] + args.version,
        content,
        count=1,
    )
    for name, dependency in dependencies.items():
        if "path" in dependency:
            content = re.sub(
                rf'(^\s*{re.escape(name)}\s*=\s*\{{[^\n]*version\s*=\s*")[^"]+',
                lambda match: match[1] + args.version,
                content,
                flags=re.MULTILINE,
            )
    tomllib.loads(content)
    manifest.write_text(content)


if __name__ == "__main__":
    main()
