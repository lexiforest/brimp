"""Assert crate layering and library-only runtime structure using Cargo's graph."""

import json
from pathlib import Path
import subprocess


def main():
    root = Path(__file__).resolve().parents[1]
    metadata = json.loads(
        subprocess.check_output(
            # Metadata includes every target, even dependencies the host build never fetched.
            ["cargo", "metadata", "--format-version=1", "--locked"],
            cwd=root,
        )
    )
    packages = {package["id"]: package for package in metadata["packages"]}
    runtime = next(package for package in packages.values() if package["name"] == "brimp-runtime")
    edges = {node["id"]: node["dependencies"] for node in metadata["resolve"]["nodes"]}
    for name, allowed in {
        "brimp-lexbor": {"thiserror", "serde", "cc"},
        "brimp-jsc": {"libc"},
    }.items():
        package = next(p for p in packages.values() if p["name"] == name)
        dependencies = {packages[dep]["name"] for dep in edges[package["id"]]}
        assert dependencies <= allowed, f"Unexpected {name} dependencies: {dependencies - allowed}"
    layers = {
        packages[dep]["name"]
        for dep in edges[runtime["id"]]
        if packages[dep]["name"].startswith("brimp-")
    }
    assert layers == {"brimp-lexbor", "brimp-jsc", "brimp-net"}, layers
    assert not any("bin" in target["kind"] for target in runtime["targets"]), (
        "The runtime must not ship a worker executable"
    )
    print("Runtime dependency boundary passed (crate layering, no executable).")


if __name__ == "__main__":
    main()
