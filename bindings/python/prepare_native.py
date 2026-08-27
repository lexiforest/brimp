#!/usr/bin/env python3
"""Download and verify the native SDKs used to build Brimp wheels."""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import sys
import tarfile
import urllib.request
from dataclasses import dataclass
from pathlib import Path


JSC_VERSION = "jsc-v2026082706"
CURL_VERSION = "v2.1.1"
JSC_BASE_URL = f"https://github.com/lexiforest/WebKit/releases/download/{JSC_VERSION}"
CURL_BASE_URL = (
    f"https://github.com/lexiforest/curl-impersonate/releases/download/{CURL_VERSION}"
)


@dataclass(frozen=True)
class Asset:
    filename: str
    sha256: str
    base_url: str

    @property
    def url(self) -> str:
        return f"{self.base_url}/{self.filename}"


@dataclass(frozen=True)
class Target:
    jsc: Asset
    curl: Asset
    windows: bool = False


TARGETS = {
    "x86_64-unknown-linux-gnu": Target(
        Asset(
            f"JavaScriptCore-{JSC_VERSION}-linux-x64.tar.gz",
            "ac72fd9de753dc0acdf2bfc389dc7bc4bd61c30f309eabeeec38930f367d3825",
            JSC_BASE_URL,
        ),
        Asset(
            "libcurl-impersonate-v2.1.1.x86_64-linux-gnu.tar.gz",
            "18b22585da3d6a58926086c65b1e662a87768ccca646e8c2a6ed03137bf948f1",
            CURL_BASE_URL,
        ),
    ),
    "aarch64-unknown-linux-gnu": Target(
        Asset(
            f"JavaScriptCore-{JSC_VERSION}-linux-arm64.tar.gz",
            "653d458e3b0103da2f4d28c17193d468fbc71983b04bb5a788c98566bfa74011",
            JSC_BASE_URL,
        ),
        Asset(
            "libcurl-impersonate-v2.1.1.aarch64-linux-gnu.tar.gz",
            "db437a38f5c694f43ae08619cb53e3ad5061b05f720f9e56ee68688c91442805",
            CURL_BASE_URL,
        ),
    ),
    "aarch64-apple-darwin": Target(
        Asset(
            f"JavaScriptCore-{JSC_VERSION}-macos-arm64.tar.gz",
            "1d5b379d96f3dccd13a62674993fc8dadf9fd1091542edb1854ccb639c22168c",
            JSC_BASE_URL,
        ),
        Asset(
            "libcurl-impersonate-v2.1.1.arm64-macos.tar.gz",
            "747ad70d1e6d302528aecd59fdf64d5c29412ec64f6217d0c7180feff1cad633",
            CURL_BASE_URL,
        ),
    ),
    "x86_64-pc-windows-msvc": Target(
        Asset(
            f"JavaScriptCore-{JSC_VERSION}-windows-x64.tar.gz",
            "28836e9afbed32c97aee0cacb9b4b27b11febc29c0c1ef13786f3a082e7a31b9",
            JSC_BASE_URL,
        ),
        Asset(
            "libcurl-impersonate-v2.1.1.x86_64-win32.tar.gz",
            "656ef0fe16393e2718d66112c7d0fcb230adfb4b0de42b0884ade598f6aea617",
            CURL_BASE_URL,
        ),
        windows=True,
    ),
}


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def download(asset: Asset, cache: Path) -> Path:
    if len(asset.sha256) != 64:
        raise RuntimeError(
            f"{asset.filename} does not have a pinned SHA-256 digest; "
            "update the release pin before downloading"
        )
    archive = cache / asset.filename
    if archive.is_file() and digest(archive) == asset.sha256:
        return archive
    archive.unlink(missing_ok=True)
    partial = archive.with_suffix(f"{archive.suffix}.partial")
    partial.unlink(missing_ok=True)
    print(f"Downloading {asset.url}", flush=True)
    with urllib.request.urlopen(asset.url) as response, partial.open("wb") as output:
        shutil.copyfileobj(response, output)
    actual = digest(partial)
    if actual != asset.sha256:
        partial.unlink(missing_ok=True)
        raise RuntimeError(
            f"SHA-256 mismatch for {asset.filename}: expected {asset.sha256}, got {actual}"
        )
    partial.replace(archive)
    return archive


def extract(archive: Path, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    with tarfile.open(archive, "r:gz") as package:
        package.extractall(destination, filter="data")


def sole_directory(directory: Path) -> Path:
    entries = list(directory.iterdir())
    if len(entries) != 1 or not entries[0].is_dir():
        raise RuntimeError(f"expected one SDK directory in {directory}")
    return entries[0]


def require_files(root: Path, paths: tuple[str, ...]) -> None:
    for relative in paths:
        path = root / relative
        if not path.is_file():
            raise RuntimeError(f"native SDK file is missing: {path}")


def append_environment(path: Path, values: dict[str, str]) -> None:
    with path.open("a", encoding="utf-8") as output:
        for name, value in values.items():
            if "\n" in value or "\r" in value:
                raise RuntimeError(f"invalid newline in {name}")
            output.write(f"{name}={value}\n")


def prepare(target_name: str, output: Path) -> dict[str, str]:
    target = TARGETS[target_name]
    cache = output / "downloads"
    cache.mkdir(parents=True, exist_ok=True)
    jsc_archive = download(target.jsc, cache)
    curl_archive = download(target.curl, cache)

    jsc_extract = output / "jsc"
    curl_extract = output / "curl"
    extract(jsc_archive, jsc_extract)
    extract(curl_archive, curl_extract)
    jsc_root = sole_directory(jsc_extract)

    jsc_lib = jsc_root / "lib"
    curl_lib = curl_extract / "lib" if target.windows else curl_extract
    runtime_dirs = [jsc_root / "bin", curl_lib] if target.windows else [jsc_lib, curl_lib]
    for directory in [jsc_lib, curl_lib, *runtime_dirs]:
        if not directory.is_dir():
            raise RuntimeError(f"native SDK directory is missing: {directory}")

    if target.windows:
        require_files(
            jsc_root,
            (
                "lib/JavaScriptCore.lib",
                "bin/JavaScriptCore.dll",
                "bin/icudt77.dll",
                "bin/icuin77.dll",
                "bin/icuuc77.dll",
            ),
        )
        require_files(
            curl_extract,
            ("lib/libcurl-impersonate_imp.lib", "lib/libcurl-impersonate.dll"),
        )
    elif "apple-darwin" in target_name:
        require_files(
            jsc_root,
            ("lib/JavaScriptCore.framework/JavaScriptCore",),
        )
        require_files(curl_extract, ("libcurl-impersonate.dylib",))
    else:
        require_files(
            jsc_root,
            (
                "lib/libJavaScriptCore.so",
                "lib/libicudata.so.77",
                "lib/libicui18n.so.77",
                "lib/libicuuc.so.77",
            ),
        )
        require_files(curl_extract, ("libcurl-impersonate.so",))

    sdk_licenses = jsc_root / "share/licenses"
    if not sdk_licenses.is_dir():
        raise RuntimeError(f"native SDK licenses are missing: {sdk_licenses}")
    bundled_licenses = Path(__file__).parent / "python/brimp/licenses/jsc-sdk"
    if bundled_licenses.exists():
        shutil.rmtree(bundled_licenses)
    shutil.copytree(sdk_licenses, bundled_licenses)

    values = {
        "BRIMP_JSC_LIB_DIR": str(jsc_lib.resolve()),
        "BRIMP_CURL_LIB_DIR": str(curl_lib.resolve()),
        "BRIMP_NATIVE_RUNTIME_DIRS": os.pathsep.join(
            str(directory.resolve()) for directory in runtime_dirs
        ),
    }
    if target.windows:
        github_path = os.environ.get("GITHUB_PATH")
        if github_path:
            with Path(github_path).open("a", encoding="utf-8") as output_file:
                for directory in runtime_dirs:
                    output_file.write(f"{directory.resolve()}\n")
        else:
            values["PATH"] = (
                values["BRIMP_NATIVE_RUNTIME_DIRS"]
                + os.pathsep
                + os.environ["PATH"]
            )
    elif "apple-darwin" in target_name:
        values["DYLD_LIBRARY_PATH"] = values["BRIMP_NATIVE_RUNTIME_DIRS"]
    else:
        values["LD_LIBRARY_PATH"] = values["BRIMP_NATIVE_RUNTIME_DIRS"]
    return values


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()
    try:
        values = prepare(arguments.target, arguments.output.resolve())
        github_environment = os.environ.get("GITHUB_ENV")
        if github_environment:
            append_environment(Path(github_environment), values)
        for name, value in values.items():
            print(f"{name}={value}")
    except (OSError, RuntimeError, tarfile.TarError) as error:
        print(f"prepare-native: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
