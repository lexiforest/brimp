"""Bundle native runtime dependencies for CLI archives and Node platform packages."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys

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


def relocate_macos(binary: Path, *, extension: bool) -> None:
    identifiers = run("otool", "-D", binary).splitlines()[1:]
    prefix = "@loader_path/native/" if extension else "@loader_path/"
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


def bundle_macos(binary: Path, jsc: Path, curl: Path, package: Path) -> None:
    native = package / "native"
    native.mkdir()
    jsc_binary = jsc / "lib/JavaScriptCore.framework/JavaScriptCore"
    curl_binary = curl / "lib/libcurl-impersonate.dylib"
    shutil.copyfile(jsc_binary.resolve(), native / "libJavaScriptCore.dylib")
    shutil.copyfile(curl_binary.resolve(), native / "libcurl-impersonate.dylib")
    shutil.copy2(binary, package / binary.name)
    for path in [native / "libJavaScriptCore.dylib", native / "libcurl-impersonate.dylib"]:
        relocate_macos(path, extension=False)
    relocate_macos(package / binary.name, extension=True)


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


def bundle_linux(binary: Path, jsc: Path, curl: Path, package: Path) -> None:
    if shutil.which("patchelf") is None:
        raise RuntimeError("patchelf is required to assemble Linux native packages")
    shutil.copy2(binary, package / binary.name)
    native = package / "native"
    native.mkdir()
    candidates = linux_candidates(jsc, curl)
    system = ldconfig_map()
    pending = list(needed(package / binary.name))
    copied = {}
    while pending:
        name = pending.pop()
        if name in copied or name in SYSTEM_LIBRARIES or name.startswith("ld-linux-"):
            continue
        source = candidates.get(name) or system.get(name)
        if source is None:
            raise RuntimeError(f"unbundled Linux dependency in {binary.name}: {name}")
        destination = native / name
        shutil.copyfile(source.resolve(), destination)
        destination.chmod(0o755)
        copied[name] = destination
        pending.extend(needed(destination))
    subprocess.run(
        ["patchelf", "--set-rpath", "$ORIGIN/native", str(package / binary.name)],
        check=True,
        capture_output=True,
    )
    for destination in copied.values():
        subprocess.run(
            ["patchelf", "--set-rpath", "$ORIGIN", str(destination)],
            check=True,
            capture_output=True,
        )


def bundle_windows(binary: Path, jsc: Path, curl: Path, package: Path, runtime: Path) -> None:
    shutil.copy2(binary, package / binary.name)
    for name in ("JavaScriptCore.dll", "icudt77.dll", "icuin77.dll", "icuuc77.dll"):
        shutil.copyfile(jsc / "bin" / name, package / name)
    shutil.copyfile(curl / "lib/libcurl-impersonate.dll", package / "libcurl-impersonate.dll")
    for name in WINDOWS_RUNTIME:
        path = runtime / name
        if not path.is_file():
            raise RuntimeError(f"Visual C++ runtime DLL is missing: {path}")
        shutil.copyfile(path, package / name)


def bundle_native(
    binary: Path, jsc: Path, curl: Path, package: Path, *, vc_runtime_dir: Path | None = None
) -> None:
    if sys.platform == "darwin":
        bundle_macos(binary, jsc, curl, package)
    elif sys.platform == "linux":
        bundle_linux(binary, jsc, curl, package)
    elif sys.platform == "win32":
        runtime = vc_runtime_dir or Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32"
        bundle_windows(binary, jsc, curl, package, runtime)
    else:
        raise RuntimeError(f"unsupported native platform: {sys.platform}")
