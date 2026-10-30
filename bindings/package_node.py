#!/usr/bin/env python3
"""Assemble relocatable Node.js platform packages and the JavaScript wrapper."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path


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

SYSTEM_LIBRARIES = {
    "libc.so.6",
    "libm.so.6",
    "libdl.so.2",
    "librt.so.1",
    "libpthread.so.0",
    "libstdc++.so.6",
    "libgcc_s.so.1",
    "libatomic.so.1",
}
WINDOWS_RUNTIME = ("msvcp140.dll", "msvcp140_2.dll", "vcruntime140.dll", "vcruntime140_1.dll")


def run(*args, **kwargs):
    return subprocess.check_output([str(arg) for arg in args], text=True, **kwargs)


def sha(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


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


def dependencies(path: Path) -> list[str]:
    return [
        line.strip().split(" (compatibility")[0]
        for line in run("otool", "-L", path).splitlines()[1:]
    ]


def macos_library(dep: str) -> str | None:
    if "JavaScriptCore" in dep:
        return "libJavaScriptCore.dylib"
    if "libcurl-impersonate" in dep:
        return "libcurl-impersonate.dylib"
    return None


def relocate_macos(binary: Path, libraries: dict[str, str], *, extension: bool) -> None:
    identifiers = run("otool", "-D", binary).splitlines()[1:]
    prefix = "@loader_path/native/" if extension else "@rpath/"
    for dep in dependencies(binary):
        if dep in identifiers:
            continue
        name = macos_library(dep)
        if name:
            subprocess.run(
                ["install_name_tool", "-change", dep, prefix + name, str(binary)],
                check=True,
                capture_output=True,
            )
        elif not dep.startswith(("/usr/lib/", "/System/Library/")):
            raise RuntimeError(f"unbundled macOS dependency in {binary.name}: {dep}")
    if not extension:
        subprocess.run(
            ["install_name_tool", "-id", "@rpath/" + binary.name, str(binary)],
            check=True,
            capture_output=True,
        )
    subprocess.run(
        ["codesign", "--force", "--sign", "-", "--timestamp=none", str(binary)],
        check=True,
        capture_output=True,
    )


def package_macos(source: Path, jsc: Path, curl: Path, package: Path) -> None:
    native = package / "native"
    native.mkdir()
    jsc_binary = jsc / "lib/JavaScriptCore.framework/JavaScriptCore"
    curl_binary = curl / "lib/libcurl-impersonate.dylib"
    shutil.copyfile(jsc_binary.resolve(), native / "libJavaScriptCore.dylib")
    shutil.copyfile(curl_binary.resolve(), native / "libcurl-impersonate.dylib")
    shutil.copyfile(source / "brimp.node", package / "brimp.node")
    libraries = {
        "libJavaScriptCore.dylib": str(native / "libJavaScriptCore.dylib"),
        "libcurl-impersonate.dylib": str(native / "libcurl-impersonate.dylib"),
    }
    for path in [native / "libJavaScriptCore.dylib", native / "libcurl-impersonate.dylib"]:
        relocate_macos(path, libraries, extension=False)
    relocate_macos(package / "brimp.node", libraries, extension=True)


def soname(path: Path) -> str:
    name = run("patchelf", "--print-soname", path).strip()
    return name or path.name


def needed(path: Path) -> list[str]:
    return [line for line in run("patchelf", "--print-needed", path).splitlines() if line]


def linux_candidates(jsc: Path, curl: Path) -> dict[str, Path]:
    candidates = {}
    for directory in (jsc / "lib", curl / "lib"):
        for path in sorted(directory.iterdir()):
            if ".so" not in path.name:
                continue
            resolved = path.resolve()
            if not resolved.is_file():
                continue
            name = soname(resolved)
            candidates[name] = resolved
    return candidates


def ldconfig_map() -> dict[str, Path]:
    executable = shutil.which("ldconfig") or "/sbin/ldconfig"
    try:
        output = run(executable, "-p")
    except (OSError, subprocess.CalledProcessError):
        output = ""
    libraries = {}
    for line in output.splitlines():
        if "=>" not in line:
            continue
        name, path = line.split("=>", 1)
        name = name.split("(", 1)[0].strip()
        path = path.strip()
        if name and Path(path).is_file():
            libraries.setdefault(name, Path(path))
    if not libraries:
        for directory in ("/usr/lib64", "/usr/lib", "/lib64", "/lib"):
            root = Path(directory)
            if root.is_dir():
                for path in root.glob("*.so*"):
                    if path.is_file():
                        libraries.setdefault(path.name, path)
    return libraries


def package_linux(source: Path, jsc: Path, curl: Path, package: Path) -> None:
    if shutil.which("patchelf") is None:
        raise RuntimeError("patchelf is required to assemble Linux Node packages")
    shutil.copyfile(source / "brimp.node", package / "brimp.node")
    native = package / "native"
    native.mkdir()
    candidates = linux_candidates(jsc, curl)
    system = ldconfig_map()
    pending = list(needed(package / "brimp.node"))
    copied = {}
    while pending:
        name = pending.pop()
        if name in copied or name in SYSTEM_LIBRARIES or name.startswith("ld-linux-"):
            continue
        source = candidates.get(name) or system.get(name)
        if source is None:
            raise RuntimeError(f"unbundled Linux dependency in brimp.node: {name}")
        destination = native / name
        shutil.copyfile(source.resolve(), destination)
        destination.chmod(0o755)
        copied[name] = destination
        pending.extend(needed(destination))
    subprocess.run(
        ["patchelf", "--set-rpath", "$ORIGIN/native", str(package / "brimp.node")],
        check=True,
        capture_output=True,
    )
    for destination in copied.values():
        subprocess.run(
            ["patchelf", "--set-rpath", "$ORIGIN", str(destination)],
            check=True,
            capture_output=True,
        )


def package_windows(source: Path, jsc: Path, curl: Path, package: Path, runtime: Path) -> None:
    shutil.copyfile(source / "brimp.node", package / "brimp.node")
    for name in ("JavaScriptCore.dll", "icudt77.dll", "icuin77.dll", "icuuc77.dll"):
        shutil.copyfile(jsc / "bin" / name, package / name)
    shutil.copyfile(curl / "lib/libcurl-impersonate.dll", package / "libcurl-impersonate.dll")
    for name in WINDOWS_RUNTIME:
        path = runtime / name
        if not path.is_file():
            raise RuntimeError(f"Visual C++ runtime DLL is missing: {path}")
        shutil.copyfile(path, package / name)


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
    if os_name == "darwin":
        package_macos(source, jsc, curl, package)
    elif os_name == "linux":
        package_linux(source, jsc, curl, package)
    else:
        package_windows(source, jsc, curl, package, runtime)
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
