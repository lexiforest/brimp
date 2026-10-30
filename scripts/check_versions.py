#!/usr/bin/env python3
"""Keep release versions and package licenses in sync."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", help="expected git tag, for example v0.1.2")
    arguments = parser.parse_args()
    with (ROOT / "Cargo.toml").open("rb") as source:
        version = tomllib.load(source)["workspace"]["package"]["version"]
    with (ROOT / "bindings/python/pyproject.toml").open("rb") as source:
        project = tomllib.load(source)["project"]
    dynamic = project.get("dynamic", [])
    if "version" not in dynamic or "version" in project:
        raise RuntimeError("bindings/python/pyproject.toml must use dynamic version")
    with (ROOT / "bindings/python/Cargo.toml").open("rb") as source:
        if tomllib.load(source)["package"].get("version") != {"workspace": True}:
            raise RuntimeError("Python binding must use the workspace version")
    with (ROOT / "bindings/node/package.json").open() as source:
        node = json.load(source)
    if node["version"] != version:
        raise RuntimeError(
            f"Node version {node['version']} does not match workspace version {version}"
        )
    for name, expected in node.get("optionalDependencies", {}).items():
        if expected != version:
            raise RuntimeError(
                f"{name} is pinned to {expected}, expected {version}"
            )
    license_text = (ROOT / "LICENSE").read_text()
    for path in (ROOT / "bindings/python/LICENSE", ROOT / "bindings/node/LICENSE"):
        if path.read_text() != license_text:
            raise RuntimeError(f"{path.relative_to(ROOT)} does not match the root LICENSE")
    if arguments.tag and arguments.tag != f"v{version}":
        raise RuntimeError(
            f"release tag {arguments.tag!r} does not match version v{version}"
        )
    print(version)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, tomllib.TOMLDecodeError) as error:
        print(f"check-versions: {error}", file=sys.stderr)
        raise SystemExit(1)
