#!/usr/bin/env python3
"""Build and validate CLI archives containing their native runtime and licenses."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile

from native_runtime import WINDOWS_RUNTIME, bundle_native, sha

ROOT = Path(__file__).resolve().parents[1]
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
)
REQUIRED_LICENSES = (
    "LICENSE",
    "licenses/lexbor-LICENSE",
    "licenses/defuddle-NOTICE.md",
    "licenses/native/curl-sdk/LICENSE",
    "licenses/native/jsc-sdk/JavaScriptCore/COPYING.LIB",
    "licenses/native/skia-LICENSE",
)


def version() -> str:
    return tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]


def archive_name(target: str) -> str:
    suffix = ".zip" if target.endswith("windows-msvc") else ".tar.gz"
    return f"brimp-v{version()}-{target}{suffix}"


def copy_licenses(native_licenses: Path, package: Path) -> None:
    shutil.copyfile(ROOT / "LICENSE", package / "LICENSE")
    licenses = package / "licenses"
    licenses.mkdir()
    for path in (ROOT / "crates/runtime/js/vendor").glob("*.LICENSE"):
        shutil.copyfile(path, licenses / path.name)
    shutil.copytree(ROOT / "assets/defuddle/0.19.3/licenses", licenses / "defuddle")
    for source, name in (
        ("crates/lexbor/vendor/lexbor/LICENSE", "lexbor-LICENSE"),
        ("crates/lexbor/vendor/lexbor/NOTICE", "lexbor-NOTICE"),
        ("assets/defuddle/0.19.3/NOTICE.md", "defuddle-NOTICE.md"),
    ):
        shutil.copyfile(ROOT / source, licenses / name)
    shutil.copytree(native_licenses, licenses / "native")


def write_archive(package: Path, destination: Path) -> None:
    files = sorted(path for path in package.rglob("*") if path.is_file())
    if destination.suffix == ".zip":
        with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for path in files:
                info = zipfile.ZipInfo(f"{package.name}/{path.relative_to(package).as_posix()}")
                info.create_system = 3
                info.external_attr = path.stat().st_mode << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(info, path.read_bytes())
    else:
        with (
            destination.open("wb") as output,
            gzip.GzipFile(filename="", fileobj=output, mode="wb", mtime=0) as compressed,
            tarfile.open(fileobj=compressed, mode="w") as archive,
        ):
            for path in files:
                info = archive.gettarinfo(
                    str(path), f"{package.name}/{path.relative_to(package).as_posix()}"
                )
                info.uid = info.gid = info.mtime = 0
                info.uname = info.gname = ""
                with path.open("rb") as content:
                    archive.addfile(info, content)


def package(binary: Path, target: str, jsc: Path, curl: Path, licenses: Path, output: Path) -> Path:
    hosts = {
        ("darwin", "arm64"): "aarch64-apple-darwin",
        ("linux", "x86_64"): "x86_64-unknown-linux-gnu",
        ("linux", "aarch64"): "aarch64-unknown-linux-gnu",
        ("win32", "AMD64"): "x86_64-pc-windows-msvc",
        ("win32", "x86_64"): "x86_64-pc-windows-msvc",
    }
    if hosts.get((sys.platform, platform.machine())) != target:
        raise RuntimeError(f"CLI packaging requires the native host for {target}")
    executable = "brimp.exe" if target.endswith("windows-msvc") else "brimp"
    if binary.name != executable or not binary.is_file():
        raise RuntimeError(f"expected a built {executable}: {binary}")
    filename = archive_name(target)
    with tempfile.TemporaryDirectory(prefix="brimp-cli-package-") as temporary:
        directory = Path(temporary) / filename.removesuffix(".tar.gz").removesuffix(".zip")
        directory.mkdir()
        bundle_native(binary, jsc, curl, directory)
        (directory / executable).chmod(0o755)
        copy_licenses(licenses, directory)
        manifest = {
            "version": version(),
            "target": target,
            "files": {
                path.relative_to(directory).as_posix(): sha(path)
                for path in sorted(directory.rglob("*"))
                if path.is_file()
            },
        }
        (directory / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        archive = output / filename
        write_archive(directory, archive)
    archive.with_name(filename + ".sha256").write_text(f"{sha(archive)}  {filename}\n")
    return archive


def validate_archive(path: Path, target: str) -> None:
    expected_root = path.name.removesuffix(".tar.gz").removesuffix(".zip") + "/"
    if path.suffix == ".zip":
        archive = zipfile.ZipFile(path)
        names = archive.namelist()
        read = archive.open
    else:
        archive = tarfile.open(path, "r:gz")
        members = archive.getmembers()
        if any(not member.isfile() for member in members):
            archive.close()
            raise RuntimeError(f"unexpected non-file entry in {path.name}")
        names = archive.getnames()
        read = archive.extractfile
    with archive:
        if len(names) != len(set(names)) or any(
            not name.startswith(expected_root) or ".." in Path(name).parts for name in names
        ):
            raise RuntimeError(f"unexpected paths in {path.name}")
        with read(expected_root + "manifest.json") as source:
            manifest = json.load(source)
        if manifest["version"] != version() or manifest["target"] != target:
            raise RuntimeError(f"incorrect version or target in {path.name}")
        files = manifest["files"]
        if {expected_root + name for name in files} | {expected_root + "manifest.json"} != set(
            names
        ):
            raise RuntimeError(f"manifest does not match archive contents: {path.name}")
        executable = "brimp.exe" if target.endswith("windows-msvc") else "brimp"
        for name in (executable, *REQUIRED_LICENSES):
            if name not in files:
                raise RuntimeError(f"{path.name} is missing {name}")
        if target.endswith("windows-msvc"):
            required_libraries = (
                "JavaScriptCore.dll",
                "libcurl-impersonate.dll",
                "icudt77.dll",
                "icuin77.dll",
                "icuuc77.dll",
                *WINDOWS_RUNTIME,
            )
        elif "linux" in target:
            required_libraries = (
                "native/libJavaScriptCore",
                "native/libcurl-impersonate",
                "native/libicudata",
                "native/libicui18n",
                "native/libicuuc",
            )
        else:
            required_libraries = ("native/libJavaScriptCore", "native/libcurl-impersonate")
        for library in required_libraries:
            if not any(name.startswith(library) for name in files):
                raise RuntimeError(f"{path.name} is missing {library}")
        for name, expected in files.items():
            with read(expected_root + name) as source:
                digest = hashlib.file_digest(source, "sha256").hexdigest()
            if digest != expected:
                raise RuntimeError(f"checksum mismatch for {name} in {path.name}")
    checksum = path.with_name(path.name + ".sha256").read_text().strip()
    if checksum != f"{sha(path)}  {path.name}":
        raise RuntimeError(f"archive checksum mismatch: {path.name}")


def validate(directory: Path, complete: bool) -> None:
    archives = sorted([*directory.glob("*.tar.gz"), *directory.glob("*.zip")])
    expected = {archive_name(target): target for target in TARGETS}
    if not archives or (complete and {path.name for path in archives} != set(expected)):
        raise RuntimeError(
            f"expected {'four' if complete else 'at least one'} CLI archives in {directory}"
        )
    for path in archives:
        if path.name not in expected:
            raise RuntimeError(f"unexpected CLI archive: {path.name}")
        validate_archive(path, expected[path.name])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        type=Path,
        default=ROOT / "target/release" / ("brimp.exe" if os.name == "nt" else "brimp"),
    )
    parser.add_argument("--target", choices=TARGETS)
    parser.add_argument("--jsc-sdk", type=Path)
    parser.add_argument("--curl-sdk", type=Path)
    parser.add_argument("--licenses", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--validate", action="store_true")
    parser.add_argument("--all", action="store_true", help="require all four platform archives")
    args = parser.parse_args()
    try:
        if args.validate:
            validate(args.out, args.all)
            print(f"validated {args.out}")
        else:
            if not all((args.target, args.jsc_sdk, args.curl_sdk, args.licenses)):
                parser.error("--target, --jsc-sdk, --curl-sdk and --licenses are required")
            args.out.mkdir(parents=True, exist_ok=True)
            print(
                package(
                    args.binary.resolve(),
                    args.target,
                    args.jsc_sdk.resolve(),
                    args.curl_sdk.resolve(),
                    args.licenses.resolve(),
                    args.out,
                )
            )
    except (OSError, RuntimeError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"package-cli: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
