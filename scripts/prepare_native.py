#!/usr/bin/env python3
"""Prepare the pinned JSC SDK, curl-impersonate release and build licenses."""

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
JSC_BASE_URL = f"https://github.com/lexiforest/WebKit/releases/download/{JSC_VERSION}"
CURL_VERSION = "v2.3.2"
CURL_BASE_URL = f"https://github.com/lexiforest/curl-impersonate/releases/download/{CURL_VERSION}"
ROOT = Path(__file__).resolve().parents[1]


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
            f"libcurl-impersonate-{CURL_VERSION}.x86_64-linux-gnu.tar.gz",
            "7716be1ec87f13ecc776c211003bd1122c345287829b1f18a494a5467c90ab1e",
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
            f"libcurl-impersonate-{CURL_VERSION}.aarch64-linux-gnu.tar.gz",
            "1cb74a3b5103df020c252f9efef5a779c87380ee6244b365a90a0e4f7086864e",
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
            f"libcurl-impersonate-{CURL_VERSION}.arm64-macos.tar.gz",
            "6c0dc841a8f6b7a8f81a0f55197e8241405fb7dcd97df180e66fc6f796c42bd5",
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
            f"libcurl-impersonate-{CURL_VERSION}.x86_64-win32.tar.gz",
            "4c416b5e50716199c4f6541b660fdada9a049cd0bdb72e92bdee4d4da27d39db",
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


def extract_sdk(archive: Path, destination: Path, *, nested: bool) -> None:
    staging = destination.with_name(destination.name + "-staging")
    extract(archive, staging)
    root = sole_directory(staging) if nested else staging
    if destination.exists():
        shutil.rmtree(destination)
    root.replace(destination)
    if staging.exists():
        shutil.rmtree(staging)


def copy_tree(source: Path, destination: Path) -> None:
    for entry in sorted(source.iterdir()):
        target = destination / entry.name
        if entry.is_dir():
            target.mkdir(parents=True, exist_ok=True)
            copy_tree(entry, target)
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(entry, target)


def require_files(root: Path, paths: tuple[str, ...]) -> None:
    for relative in paths:
        path = root / relative
        if not path.is_file():
            raise RuntimeError(f"native SDK file is missing: {path}")


def normalize_curl_root(root: Path) -> None:
    """Place curl's shared libraries under lib/ for Linux and macOS archives."""
    if (root / "lib").is_dir():
        return
    library = root / "lib"
    library.mkdir()
    for entry in list(root.iterdir()):
        if entry.name in {"include", "lib"}:
            continue
        if entry.name.startswith("libcurl-impersonate"):
            shutil.move(str(entry), library / entry.name)


def append_environment(path: Path, values: dict[str, str]) -> None:
    with path.open("a", encoding="utf-8") as output:
        for name, value in values.items():
            if "\n" in value or "\r" in value:
                raise RuntimeError(f"invalid newline in {name}")
            output.write(f"{name}={value}\n")


def collect_licenses(
    jsc_root: Path, curl_root: Path | None, output: Path, *, required: bool
) -> Path:
    licenses = output / "licenses"
    if licenses.exists():
        shutil.rmtree(licenses)
    jsc_licenses = jsc_root / "share/licenses"
    if not jsc_licenses.is_dir():
        raise RuntimeError(f"native SDK licenses are missing: {jsc_licenses}")
    copy_tree(jsc_licenses, licenses / "jsc-sdk")
    if curl_root is not None:
        curl_licenses = licenses / "curl-sdk"
        curl_licenses.mkdir(parents=True)
        files = sorted(curl_root.glob("LICENSE*"))
        if required and not files:
            raise RuntimeError(f"curl-impersonate licenses are missing: {curl_root}")
        for path in files:
            shutil.copyfile(path, curl_licenses / path.name)
    skia = ROOT / "bindings/licenses/skia-LICENSE"
    if not skia.is_file():
        raise RuntimeError(f"Skia license is missing: {skia}")
    shutil.copyfile(skia, licenses / "skia-LICENSE")
    return licenses.resolve()


def prepare(
    target_name: str,
    output: Path,
    curl_sdk: Path | None = None,
    *,
    curl_release: bool = False,
) -> dict[str, str]:
    if curl_sdk is not None and curl_release:
        raise RuntimeError("choose either a curl SDK or the pinned curl release")
    target = TARGETS[target_name]
    cache = output / "downloads"
    cache.mkdir(parents=True, exist_ok=True)

    jsc_archive = download(target.jsc, cache)
    extract_sdk(jsc_archive, output / "jsc", nested=True)
    jsc_root = output / "jsc"
    jsc_lib = jsc_root / "lib"

    curl_root = None
    if curl_release:
        curl_archive = download(target.curl, cache)
        extract_sdk(curl_archive, output / "curl", nested=False)
        curl_root = output / "curl"
    elif curl_sdk is not None:
        curl_root = Path(curl_sdk).resolve()
    if curl_root is not None:
        normalize_curl_root(curl_root)
        require_files(curl_root, ("include/curl-impersonate/webcrypto.h",))
    curl_lib = curl_root / "lib" if curl_root is not None else None

    runtime_dirs = [jsc_lib]
    if target.windows:
        runtime_dirs = [jsc_root / "bin"]
        if curl_lib is not None:
            runtime_dirs.append(curl_lib)
    elif curl_lib is not None:
        runtime_dirs.append(curl_lib)
    for directory in [jsc_lib, *runtime_dirs]:
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
        if curl_lib is not None:
            require_files(
                curl_lib,
                ("libcurl-impersonate_imp.lib", "libcurl-impersonate.dll"),
            )
    elif "apple-darwin" in target_name:
        require_files(
            jsc_root,
            ("lib/JavaScriptCore.framework/JavaScriptCore",),
        )
        if curl_lib is not None:
            require_files(curl_lib, ("libcurl-impersonate.dylib",))
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
        if curl_lib is not None:
            require_files(curl_lib, ("libcurl-impersonate.so",))

    licenses = collect_licenses(jsc_root, curl_root, output, required=curl_release)
    values = {
        "BRIMP_JSC_SDK": str(jsc_root.resolve()),
        "BRIMP_JSC_LIB_DIR": str(jsc_lib.resolve()),
        "BRIMP_LICENSE_DIR": str(licenses),
        "BRIMP_NATIVE_RUNTIME_DIRS": os.pathsep.join(
            str(directory.resolve()) for directory in runtime_dirs
        ),
    }
    if curl_root is not None and curl_lib is not None:
        values["BRIMP_CURL_SDK"] = str(curl_root.resolve())
        values["BRIMP_CURL_LIB_DIR"] = str(curl_lib.resolve())
    if target.windows:
        github_path = os.environ.get("GITHUB_PATH")
        if github_path:
            with Path(github_path).open("a", encoding="utf-8") as output_file:
                for directory in runtime_dirs:
                    output_file.write(f"{directory.resolve()}\n")
        else:
            values["PATH"] = values["BRIMP_NATIVE_RUNTIME_DIRS"] + os.pathsep + os.environ["PATH"]
    elif "apple-darwin" in target_name:
        values["DYLD_LIBRARY_PATH"] = values["BRIMP_NATIVE_RUNTIME_DIRS"]
        # WebKit SDK frameworks may retain the system framework install name.
        # Select the downloaded framework at runtime as well as at link time.
        values["DYLD_FRAMEWORK_PATH"] = str(jsc_lib.resolve())
    else:
        values["LD_LIBRARY_PATH"] = values["BRIMP_NATIVE_RUNTIME_DIRS"]
    return values


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--curl-release",
        action="store_true",
        help="download the pinned curl-impersonate binary release",
    )
    parser.add_argument(
        "--curl-sdk",
        type=Path,
        help="installed curl-impersonate SDK prefix with the BoringSSL WebCrypto interface",
    )
    arguments = parser.parse_args()
    try:
        values = prepare(
            arguments.target,
            arguments.output.resolve(),
            arguments.curl_sdk.resolve() if arguments.curl_sdk else None,
            curl_release=arguments.curl_release,
        )
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
