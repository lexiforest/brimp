#!/usr/bin/env python3
"""Assemble relocatable Node.js platform packages and the JavaScript wrapper."""

from __future__ import annotations

import argparse
import gzip
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

from native_runtime import bundle_native, sha

ROOT = Path(__file__).resolve().parents[1]
MAIN_FILES = (
    "index.js",
    "index.d.ts",
    "plugins.js",
    "values.js",
    "README.md",
    "LICENSE",
    "bundled",
)

PLATFORMS = {
    ("darwin", "arm64"): ("@brimp/brimp-darwin-arm64", "darwin", "arm64", "darwin-arm64"),
    ("linux", "x86_64"): ("@brimp/brimp-linux-x64-gnu", "linux", "x64", "linux-x64"),
    ("linux", "aarch64"): ("@brimp/brimp-linux-arm64-gnu", "linux", "arm64", "linux-arm64"),
    ("win32", "AMD64"): ("@brimp/brimp-win32-x64", "win32", "x64", "win32-x64"),
    ("win32", "x86_64"): ("@brimp/brimp-win32-x64", "win32", "x64", "win32-x64"),
}


def npm_archive(package: Path, output: Path) -> None:
    with (
        output.open("wb") as stream,
        gzip.GzipFile(filename="", fileobj=stream, mode="wb", mtime=0) as compressed,
        tarfile.open(fileobj=compressed, mode="w") as archive,
    ):
        for path in sorted(package.rglob("*")):
            if path.is_file():
                info = archive.gettarinfo(
                    str(path), "package/" + path.relative_to(package).as_posix()
                )
                info.uid = info.gid = info.mtime = 0
                info.uname = info.gname = ""
                with path.open("rb") as content:
                    archive.addfile(info, content)


def copy_licenses(source: Path, package: Path) -> None:
    shutil.copyfile(source / "LICENSE", package / "LICENSE")
    bundled = source / "bundled/licenses"
    if bundled.is_dir():
        shutil.copytree(bundled, package / "licenses")


def package_json(source: dict, name: str, os_name: str | None, cpu: str | None) -> dict:
    package = {
        "name": name,
        "version": source["version"],
        "description": source["description"],
        "license": source["license"],
        "main": "brimp.node",
        "files": ["brimp.node", "LICENSE", "licenses", "native"],
        "engines": source["engines"],
    }
    if os_name is not None:
        package["os"] = [os_name]
        package["cpu"] = [cpu]
        if os_name == "linux":
            package["libc"] = ["glibc"]
    return package


def replace_platform_files(package: dict, os_name: str | None) -> dict:
    if os_name == "win32":
        package["files"] = ["brimp.node", "LICENSE", "licenses", "*.dll"]
    return package


def package_main(source: Path, output: Path) -> Path:
    package = output / "main"
    if package.exists():
        shutil.rmtree(package)
    package.mkdir()
    for name in MAIN_FILES:
        path = source / name
        if path.is_dir():
            shutil.copytree(path, package / name)
        else:
            shutil.copyfile(path, package / name)
    manifest = json.loads((source / "package.json").read_text())
    manifest["files"] = list(MAIN_FILES)
    (package / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    archive = output / f"brimp-npm-{manifest['version']}.tgz"
    npm_archive(package, archive)
    return archive


def platform_target() -> tuple[str, str, str, str]:
    key = (sys.platform, platform.machine())
    if key not in PLATFORMS:
        raise RuntimeError(f"unsupported Node.js platform: {sys.platform}-{platform.machine()}")
    return PLATFORMS[key]


def package_platform(source: Path, jsc: Path, curl: Path, output: Path, runtime: Path) -> Path:
    name, os_name, cpu, suffix = platform_target()
    package = output / f"platform-{suffix}"
    if package.exists():
        shutil.rmtree(package)
    package.mkdir()
    copy_licenses(source, package)
    bundle_native(source / "brimp.node", jsc, curl, package, vc_runtime_dir=runtime)
    manifest = package_json(json.loads((source / "package.json").read_text()), name, os_name, cpu)
    manifest = replace_platform_files(manifest, os_name)
    (package / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    archive = output / f"brimp-{suffix}-{manifest['version']}.tgz"
    npm_archive(package, archive)
    return archive


def validate(directory: Path, *, complete: bool = False) -> None:
    expected_version = json.loads((ROOT / "bindings/node/package.json").read_text())["version"]
    archives = sorted(directory.glob("*.tgz"))
    main = [path for path in archives if path.name.startswith("brimp-npm-")]
    platform_packages = [path for path in archives if path not in main]
    if complete and (len(main) != 1 or len(platform_packages) != 4):
        raise RuntimeError(
            f"expected one main and four platform packages in {directory}: {[path.name for path in archives]}"
        )
    if not complete and (len(main) > 1 or not platform_packages):
        raise RuntimeError(
            f"expected platform packages in {directory}: {[path.name for path in archives]}"
        )
    for archive in archives:
        with tarfile.open(archive, "r:gz") as package:
            names = package.getnames()
            manifest = json.load(package.extractfile("package/package.json"))
            if manifest["version"] != expected_version:
                raise RuntimeError(
                    f"{archive.name} has version {manifest['version']}, expected {expected_version}"
                )
            if archive == main[0]:
                if manifest["name"] != "@brimp/brimp" or "package/brimp.node" in names:
                    raise RuntimeError(f"invalid main package: {archive.name}")
                if "package/bundled/catalog.json" not in names:
                    raise RuntimeError(f"main package is missing bundled plugins: {archive.name}")
                if "package/LICENSE" not in names:
                    raise RuntimeError(f"main package is missing LICENSE: {archive.name}")
                continue
            if manifest["name"] not in {value[0] for value in PLATFORMS.values()}:
                raise RuntimeError(f"invalid platform package name in {archive.name}")
            if "package/brimp.node" not in names:
                raise RuntimeError(f"platform package is missing brimp.node: {archive.name}")
            if "package/LICENSE" not in names:
                raise RuntimeError(f"platform package is missing LICENSE: {archive.name}")
            if not any(
                name.startswith("package/native/") or name.endswith(".dll") for name in names
            ):
                raise RuntimeError(f"platform package is missing native libraries: {archive.name}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "bindings/node")
    parser.add_argument("--jsc-sdk", type=Path)
    parser.add_argument("--curl-sdk", type=Path)
    parser.add_argument("--vc-runtime-dir", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--main", action="store_true", help="also package the JavaScript wrapper")
    parser.add_argument("--package", action="store_true", help="package the native platform module")
    parser.add_argument("--validate", action="store_true", help="validate packages in --out")
    parser.add_argument("--all", action="store_true", help="require all four platform packages")
    arguments = parser.parse_args()
    try:
        arguments.out.mkdir(parents=True, exist_ok=True)
        if arguments.validate:
            validate(arguments.out, complete=arguments.all)
            print(f"validated {arguments.out}")
            return 0
        if not arguments.main and not arguments.package:
            arguments.package = True
        if arguments.main:
            archive = package_main(arguments.source, arguments.out)
            archive.with_name(archive.name + ".sha256").write_text(
                f"{sha(archive)}  {archive.name}\n"
            )
            print(archive)
        if arguments.package:
            if arguments.jsc_sdk is None or arguments.curl_sdk is None:
                raise RuntimeError("--jsc-sdk and --curl-sdk are required for platform packages")
            runtime = (
                arguments.vc_runtime_dir
                or Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32"
            )
            archive = package_platform(
                arguments.source,
                arguments.jsc_sdk.resolve(),
                arguments.curl_sdk.resolve(),
                arguments.out,
                runtime,
            )
            archive.with_name(archive.name + ".sha256").write_text(
                f"{sha(archive)}  {archive.name}\n"
            )
            print(archive)
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"package-node: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
