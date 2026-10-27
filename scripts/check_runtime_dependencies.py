"""Assert the DOM runtime's transitive dependency boundary using Cargo's graph."""
import json
from pathlib import Path
import subprocess


def main():
    root = Path(__file__).resolve().parents[1]
    metadata = json.loads(subprocess.check_output(
        # Metadata includes every target, even dependencies the host build never fetched.
        ["cargo", "metadata", "--format-version=1", "--locked"],
        cwd=root,
    ))
    packages = {package["id"]: package for package in metadata["packages"]}
    runtime = next(package for package in packages.values()
                   if package["name"] == "brimp-runtime")
    edges = {node["id"]: node["dependencies"] for node in metadata["resolve"]["nodes"]}
    for name, allowed in {
        "brimp-lexbor": {"thiserror", "serde", "cc"},
        "brimp-jsc": {"libc"},
    }.items():
        package = next(p for p in packages.values() if p["name"] == name)
        dependencies = {packages[dep]["name"] for dep in edges[package["id"]]}
        assert dependencies <= allowed, f"Unexpected {name} dependencies: {dependencies - allowed}"
    layers = {packages[dep]["name"] for dep in edges[runtime["id"]]
              if packages[dep]["name"].startswith("brimp-")}
    assert layers == {"brimp-lexbor", "brimp-jsc", "brimp-net"}, layers
    visited, pending = set(), [runtime["id"]]
    while pending:
        package = pending.pop()
        if package not in visited:
            visited.add(package)
            pending.extend(edges[package])
    names = sorted(packages[package]["name"] for package in visited)
    forbidden = ("brimp-protocol", "blitz", "stylo", "style", "skia",
                 "vello", "fontique", "parley", "html5ever", "xml5ever",
                 "font-kit", "fontdb", "rustybuzz", "swash", "openssl")
    found = [name for name in names if any(
        name == prefix or name.startswith(prefix + "-") for prefix in forbidden
    ) or name == "brimp"]
    assert not found, f"Forbidden runtime dependencies: {found}"
    assert not any("bin" in target["kind"] for target in runtime["targets"]), \
        "The runtime must not ship a worker executable"
    print(f"Runtime dependency boundary passed ({len(names)} packages, no executable).")


if __name__ == "__main__":
    main()
