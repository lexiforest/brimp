#!/usr/bin/env python3
"""Validate that a Brimp wheel is complete and has the expected platform tag."""

from __future__ import annotations

import argparse
import re
import tomllib
import zipfile
from pathlib import Path


PLATFORMS = {
    "x86_64-unknown-linux-gnu": "manylinux_2_28_x86_64",
    "aarch64-unknown-linux-gnu": "manylinux_2_28_aarch64",
    "aarch64-apple-darwin": "macosx_11_0_arm64",
    "x86_64-pc-windows-msvc": "win_amd64",
}

REQUIRED_LIBRARIES = {
    "x86_64-unknown-linux-gnu": (
        "libJavaScriptCore",
        "libcurl-impersonate",
        "libicudata",
        "libicui18n",
        "libicuuc",
        "libfontconfig",
        "libatomic",
    ),
    "aarch64-unknown-linux-gnu": (
        "libJavaScriptCore",
        "libcurl-impersonate",
        "libicudata",
        "libicui18n",
        "libicuuc",
        "libfontconfig",
        "libatomic",
    ),
    "aarch64-apple-darwin": (
        "JavaScriptCore",
        "libcurl-impersonate",
    ),
    "x86_64-pc-windows-msvc": (
        "JavaScriptCore",
        "libcurl-impersonate",
        "icudt77",
        "icuin77",
        "icuuc77",
    ),
}

NATIVE_LIBRARY_PREFIX = {
    "x86_64-unknown-linux-gnu": "brimp.libs/",
    "aarch64-unknown-linux-gnu": "brimp.libs/",
    "aarch64-apple-darwin": "brimp.dylibs/",
    "x86_64-pc-windows-msvc": "brimp.libs/",
}

REQUIRED_LICENSES = (
    "brimp/licenses/curl-impersonate-LICENSE",
    "brimp/licenses/jsc-sdk/JavaScriptCore/COPYING.LIB",
)


def project_version(root: Path) -> str:
    with (root / "bindings/python/pyproject.toml").open("rb") as source:
        python_version = tomllib.load(source)["project"]["version"]
    with (root / "Cargo.toml").open("rb") as source:
        rust_version = tomllib.load(source)["workspace"]["package"]["version"]
    if python_version != rust_version:
        raise RuntimeError(
            f"Python version {python_version} does not match Rust version {rust_version}"
        )
    return python_version


def find_wheel(directory: Path, target: str, version: str) -> Path:
    platform = PLATFORMS[target]
    expected = re.compile(
        rf"^brimp-{re.escape(version)}-cp310-abi3-{re.escape(platform)}\.whl$"
    )
    wheels = [path for path in directory.glob("*.whl") if expected.match(path.name)]
    if len(wheels) != 1:
        raise RuntimeError(
            f"expected one brimp {version} cp310-abi3 {platform} wheel in "
            f"{directory}, found {[path.name for path in wheels]}"
        )
    return wheels[0]


def validate(wheel: Path, target: str) -> None:
    if wheel.stat().st_size > 100_000_000:
        raise RuntimeError(f"{wheel.name} exceeds PyPI's 100 MB file limit")
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
    extension_suffix = ".pyd" if target.endswith("windows-msvc") else ".so"
    if not any(
        name.startswith("brimp/_brimp") and name.endswith(extension_suffix)
        for name in names
    ):
        raise RuntimeError(f"{wheel.name} does not contain the Brimp extension")
    if any(
        name.lower().endswith(".pdb") or ".dsym/" in name.lower()
        for name in names
    ):
        raise RuntimeError(f"{wheel.name} contains debug symbols")
    native_names = [
        name for name in names if name.startswith(NATIVE_LIBRARY_PREFIX[target])
    ]
    for required in REQUIRED_LIBRARIES[target]:
        if not any(required.lower() in name.lower() for name in native_names):
            raise RuntimeError(
                f"{wheel.name} does not bundle {required}; found {native_names}"
            )
    for required in REQUIRED_LICENSES:
        if required not in names:
            raise RuntimeError(f"{wheel.name} does not bundle {required}")
    if "linux-gnu" in target and not any(
        name.startswith("brimp/licenses/jsc-sdk/icu/") for name in names
    ):
        raise RuntimeError(f"{wheel.name} does not bundle the ICU license")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True, choices=PLATFORMS)
    parser.add_argument("--directory", required=True, type=Path)
    parser.add_argument("--tag")
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    version = project_version(root)
    if arguments.tag and arguments.tag != f"v{version}":
        raise RuntimeError(
            f"release tag {arguments.tag!r} does not match project version v{version}"
        )
    wheel = find_wheel(arguments.directory, arguments.target, version)
    validate(wheel, arguments.target)
    print(wheel)


if __name__ == "__main__":
    main()
