"""Validate the complete WebKit release set before publishing any package."""

import argparse
import hashlib
import json
import tarfile
import zipfile
from pathlib import Path

from download import CONFIG, digest
from package import version


def validate(directory):
    config = json.loads(CONFIG.read_text())
    wheels = list(directory.glob("*.whl"))
    if len(wheels) != 2:
        raise RuntimeError("Expected exactly two WebKit wheels")
    targets = set()
    payloads = {}
    for wheel in wheels:
        with zipfile.ZipFile(wheel) as archive:
            info = json.loads(archive.read("brimp_webkit/runtime.json"))
            if (
                info["development"]
                or info["version"] != version()
                or info["sdk_tag"] != config["tag"]
            ):
                raise RuntimeError("WebKit release provenance/version mismatch")
            if (
                hashlib.sha256(archive.read("brimp_webkit/runtime.tar.xz")).hexdigest()
                != info["sha256"]
            ):
                raise RuntimeError("WebKit wheel payload checksum mismatch")
            target = info["target"]
            if target in targets:
                raise RuntimeError("Duplicate WebKit wheel target")
            targets.add(target)
            payloads[target] = info
            if target == "linux-x64" and not wheel.name.endswith("-manylinux_2_39_x86_64.whl"):
                raise RuntimeError(
                    "Linux WebKit wheel must be auditwheel-repaired for manylinux 2.39"
                )
            if target == "darwin-arm64" and "-macosx_" not in wheel.name:
                raise RuntimeError("macOS WebKit wheel must declare its deployment target")
            if not wheel.name.startswith(f"brimp_webkit-{version()}-py3-none-"):
                raise RuntimeError("Unexpected WebKit wheel name/version")
    if targets != {"darwin-arm64", "linux-x64"}:
        raise RuntimeError("WebKit release requires macOS arm64 and Linux x64")
    if len({info["sdk_commit"] for info in payloads.values()}) != 1:
        raise RuntimeError("WebKit platforms must use the same engine source revision")
    for target, info in payloads.items():
        archive = directory / f"brimp-webkit-{version()}-{target}.tar.xz"
        if (
            digest(archive) != info["sha256"]
            or json.loads(archive.with_suffix(".json").read_text()) != info
        ):
            raise RuntimeError("Standalone WebKit archive differs from the package payload")
    packages = list(directory.glob("*.tgz"))
    if len(packages) != 3:
        raise RuntimeError("Expected the Node wrapper and two WebKit platform packages")
    names = set()
    for package in packages:
        with tarfile.open(package) as archive:
            manifest = json.load(archive.extractfile("package/package.json"))
            if manifest["version"] != version():
                raise RuntimeError("WebKit npm version mismatch")
            name = manifest["name"]
            names.add(name)
            if name == "@brimp/webkit":
                if manifest["optionalDependencies"] != {
                    f"@brimp/webkit-{target}": version() for target in targets
                }:
                    raise RuntimeError("WebKit npm wrapper dependency mismatch")
            else:
                target = name.removeprefix("@brimp/webkit-")
                info = json.load(archive.extractfile("package/runtime.json"))
                if info != payloads[target]:
                    raise RuntimeError("Python and Node must ship identical WebKit payloads")
                with archive.extractfile("package/runtime.tar.xz") as source:
                    if hashlib.file_digest(source, "sha256").hexdigest() != info["sha256"]:
                        raise RuntimeError("WebKit npm payload checksum mismatch")
    if names != {"@brimp/webkit", "@brimp/webkit-darwin-arm64", "@brimp/webkit-linux-x64"}:
        raise RuntimeError("Unexpected WebKit npm packages")
    print("Validated both WebKit wheels and all three npm packages")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    validate(parser.parse_args().directory)
