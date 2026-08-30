#!/usr/bin/env python3
"""Build a relocatable archive around an already-built Brimp CLI binary."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path


TARGETS = {
    "x86_64-unknown-linux-gnu": "linux",
    "aarch64-unknown-linux-gnu": "linux",
    "aarch64-apple-darwin": "macos",
    "x86_64-pc-windows-msvc": "windows",
}

LINUX_SYSTEM_LIBRARIES = {
    "ld-linux-aarch64.so.1",
    "ld-linux-x86-64.so.2",
    "libc.so.6",
    "libdl.so.2",
    "libm.so.6",
    "libpthread.so.0",
    "libresolv.so.2",
    "librt.so.1",
    "libutil.so.1",
}


def run(*command: str, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(
        command,
        check=True,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return result.stdout


def project_version(root: Path) -> str:
    with (root / "Cargo.toml").open("rb") as source:
        return tomllib.load(source)["workspace"]["package"]["version"]


def copy_licenses(root: Path, package: Path, jsc_library_dir: Path) -> None:
    licenses = package / "licenses"
    licenses.mkdir()
    shutil.copy2(root / "LICENSE", licenses / "brimp-LICENSE")
    shutil.copy2(
        root / "bindings/python/python/brimp/licenses/curl-impersonate-LICENSE",
        licenses,
    )
    defuddle = root / "crates/web-runtime/vendor/defuddle/0.19.3"
    shutil.copytree(defuddle / "licenses", licenses / "defuddle")
    shutil.copy2(defuddle / "NOTICE.md", licenses / "defuddle-NOTICE.md")
    jsc_licenses = jsc_library_dir.parent / "share/licenses"
    if not jsc_licenses.is_dir():
        raise RuntimeError(f"JavaScriptCore licenses are missing: {jsc_licenses}")
    shutil.copytree(jsc_licenses, licenses / "jsc-sdk", symlinks=True)
    shutil.copy2(root / "crates/cli/README.md", package / "README.md")


def linux_dependencies(path: Path) -> dict[str, Path]:
    dependencies: dict[str, Path] = {}
    for line in run("ldd", str(path)).splitlines():
        line = line.strip()
        if not line or line.startswith("linux-vdso"):
            continue
        if "=> not found" in line:
            raise RuntimeError(f"unresolved dependency for {path}: {line}")
        match = re.match(r"(\S+)\s+=>\s+(\S+)\s+\(", line)
        if match:
            dependencies[match.group(1)] = Path(match.group(2))
            continue
        match = re.match(r"(/\S+)\s+\(", line)
        if match:
            library = Path(match.group(1))
            dependencies[library.name] = library
    return dependencies


def copy_linux_package_licenses(library: Path, package: Path) -> None:
    owner = subprocess.run(
        ("rpm", "-qf", "--qf", "%{NAME}", str(library)),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    if owner.returncode:
        return
    package_name = owner.stdout.strip()
    listed = run("rpm", "-ql", package_name).splitlines()
    license_files = [
        Path(path)
        for path in listed
        if path.startswith("/usr/share/licenses/") and Path(path).is_file()
    ]
    if not license_files:
        raise RuntimeError(f"RPM package {package_name} has no installed license files")
    destination = package / "licenses/system" / package_name
    destination.mkdir(parents=True, exist_ok=True)
    for license_file in license_files:
        shutil.copy2(license_file, destination / license_file.name)


def package_linux(binary: Path, package: Path) -> None:
    executable = package / "brimp"
    libraries = package / "lib"
    libraries.mkdir()
    shutil.copy2(binary, executable)

    queue = [executable]
    copied: dict[str, Path] = {}
    while queue:
        source = queue.pop()
        for name, dependency in linux_dependencies(source).items():
            if name in LINUX_SYSTEM_LIBRARIES:
                continue
            destination = libraries / name
            previous = copied.get(name)
            resolved = dependency.resolve()
            if previous:
                if previous != resolved:
                    raise RuntimeError(
                        f"conflicting Linux libraries named {name}: {previous}, {resolved}"
                    )
                continue
            shutil.copy2(resolved, destination)
            copy_linux_package_licenses(resolved, package)
            copied[name] = resolved
            queue.append(destination)

    run("patchelf", "--set-rpath", "$ORIGIN/lib", str(executable))
    for library in libraries.iterdir():
        run("patchelf", "--set-rpath", "$ORIGIN", str(library))
    if not copied:
        raise RuntimeError("Linux package did not collect any native libraries")


def otool_dependencies(path: Path) -> list[str]:
    return [
        line.strip().split(maxsplit=1)[0]
        for line in run("otool", "-L", str(path)).splitlines()[1:]
        if line.strip()
    ]


def package_macos(
    binary: Path,
    package: Path,
    jsc_library_dir: Path,
    curl_library_dir: Path,
) -> None:
    executable = package / "brimp"
    frameworks = package / "Frameworks"
    libraries = package / "lib"
    frameworks.mkdir()
    libraries.mkdir()
    shutil.copy2(binary, executable)

    framework = jsc_library_dir / "JavaScriptCore.framework"
    if not framework.is_dir():
        raise RuntimeError(f"JavaScriptCore framework is missing: {framework}")
    shutil.copytree(framework, frameworks / framework.name, symlinks=True)
    curl_candidates = sorted(curl_library_dir.glob("libcurl-impersonate*.dylib"))
    curl_source = next((path for path in curl_candidates if path.is_file()), None)
    if not curl_source:
        raise RuntimeError(f"curl-impersonate dylib is missing: {curl_library_dir}")
    shutil.copy2(curl_source.resolve(), libraries / "libcurl-impersonate.4.dylib")

    dependencies = otool_dependencies(executable)
    jsc_dependency = next(
        (dependency for dependency in dependencies if "JavaScriptCore.framework" in dependency),
        None,
    )
    curl_dependency = next(
        (dependency for dependency in dependencies if "libcurl-impersonate" in dependency),
        None,
    )
    if not jsc_dependency or not curl_dependency:
        raise RuntimeError(f"CLI native dependencies are incomplete: {dependencies}")
    run(
        "install_name_tool",
        "-change",
        jsc_dependency,
        "@rpath/JavaScriptCore.framework/JavaScriptCore",
        "-change",
        curl_dependency,
        "@rpath/libcurl-impersonate.4.dylib",
        "-add_rpath",
        "@executable_path/Frameworks",
        "-add_rpath",
        "@executable_path/lib",
        str(executable),
    )
    load_commands = run("otool", "-l", str(executable))
    if f"path {jsc_library_dir} " in load_commands:
        run("install_name_tool", "-delete_rpath", str(jsc_library_dir), str(executable))
    run("codesign", "--force", "--sign", "-", "--timestamp=none", str(executable))


def package_windows(
    binary: Path,
    package: Path,
    jsc_library_dir: Path,
    curl_library_dir: Path,
) -> None:
    shutil.copy2(binary, package / "brimp.exe")
    sources = [
        jsc_library_dir.parent / "bin/JavaScriptCore.dll",
        jsc_library_dir.parent / "bin/icudt77.dll",
        jsc_library_dir.parent / "bin/icuin77.dll",
        jsc_library_dir.parent / "bin/icuuc77.dll",
        curl_library_dir / "libcurl-impersonate.dll",
    ]
    for source in sources:
        if not source.is_file():
            raise RuntimeError(f"Windows runtime library is missing: {source}")
        shutil.copy2(source, package)


def validate(package: Path, platform: str, forbidden_roots: list[Path]) -> None:
    executable = package / ("brimp.exe" if platform == "windows" else "brimp")
    environment = os.environ.copy()
    for name in ("DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH", "BRIMP_NATIVE_RUNTIME_DIRS"):
        environment.pop(name, None)
    forbidden = [root.resolve() for root in forbidden_roots]
    environment["PATH"] = os.pathsep.join(
        entry
        for entry in environment.get("PATH", "").split(os.pathsep)
        if entry and not any(Path(entry).resolve().is_relative_to(root) for root in forbidden)
    )
    run(str(executable), "doctor", env=environment)

    inspected = b""
    files = [executable]
    if platform == "linux":
        files.extend((package / "lib").iterdir())
    for path in files:
        inspected += path.read_bytes()
    for root in forbidden:
        encoded = str(root).encode()
        if encoded and encoded in inspected:
            raise RuntimeError(f"package retains build-machine path {root}")


def archive(package: Path, output: Path, platform: str) -> list[Path]:
    output.mkdir(parents=True, exist_ok=True)
    suffix = ".zip" if platform == "windows" else ".tar.gz"
    destination = output / f"{package.name}{suffix}"
    if platform == "windows":
        with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as bundle:
            for path in package.rglob("*"):
                if path.is_file():
                    bundle.write(path, Path(package.name) / path.relative_to(package))
    else:
        with tarfile.open(destination, "w:gz", dereference=False) as bundle:
            bundle.add(package, arcname=package.name)
    checksum = hashlib.sha256(destination.read_bytes()).hexdigest()
    checksum_path = destination.with_name(f"{destination.name}.sha256")
    checksum_path.write_text(f"{checksum}  {destination.name}\n", encoding="ascii")
    return [destination, checksum_path]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--jsc-lib-dir", type=Path)
    parser.add_argument("--curl-lib-dir", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--forbid-path", action="append", default=[], type=Path)
    arguments = parser.parse_args()

    root = Path(__file__).resolve().parents[2]
    platform = TARGETS[arguments.target]
    binary = arguments.binary or (
        root
        / "target"
        / arguments.target
        / "release"
        / ("brimp.exe" if platform == "windows" else "brimp")
    )
    jsc_library_dir = arguments.jsc_lib_dir or Path(
        os.environ["BRIMP_JSC_LIB_DIR"]
    )
    curl_library_dir = arguments.curl_lib_dir or Path(
        os.environ["BRIMP_CURL_LIB_DIR"]
    )
    version = project_version(root)
    package_name = f"brimp-v{version}-{arguments.target}"
    with tempfile.TemporaryDirectory(prefix="brimp-cli-release-") as temporary:
        package = Path(temporary) / package_name
        package.mkdir()
        copy_licenses(root, package, jsc_library_dir)
        if platform == "linux":
            package_linux(binary, package)
        elif platform == "macos":
            package_macos(
                binary,
                package,
                jsc_library_dir,
                curl_library_dir,
            )
        else:
            package_windows(
                binary,
                package,
                jsc_library_dir,
                curl_library_dir,
            )
        validate(
            package,
            platform,
            [jsc_library_dir, curl_library_dir, *arguments.forbid_path],
        )
        created = archive(package, arguments.output, platform)
    for path in created:
        print(path)


if __name__ == "__main__":
    main()
