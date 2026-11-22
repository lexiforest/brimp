"""Build brimp-webkit wheels and @brimp/webkit packages from a verified SDK."""

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

from build import verify
from download import CONFIG, digest, verify_sdk

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ROOT / "bindings/webkit"


def run(*command, **kwargs):
    subprocess.run([str(part) for part in command], check=True, **kwargs)


def version():
    return json.loads(CONFIG.read_text())["version"]


def metadata(root, tag):
    info = root / f"brimp_webkit-{version()}.dist-info"
    info.mkdir(exist_ok=True)
    (info / "METADATA").write_text(
        f"Metadata-Version: 2.4\nName: brimp-webkit\nVersion: {version()}\n"
        "Summary: Bundled WebKit engine for Brimp\nRequires-Python: >=3.11.4\n"
        "License: See bundled third-party notices\n"
        "Project-URL: Source, https://github.com/lexiforest/brimp\n\n"
        "Native engine with bundled third-party notices. Linux requires Xvfb or a display, "
        "D-Bus, bubblewrap, xdg-dbus-proxy, fonts and graphics drivers.\n"
    )
    (info / "WHEEL").write_text(
        f"Wheel-Version: 1.0\nGenerator: brimp\nRoot-Is-Purelib: false\nTag: py3-none-{tag}\n"
    )
    return info


def wheel(root, output, tag):
    from wheel.wheelfile import WheelFile

    path = output / f"brimp_webkit-{version()}-py3-none-{tag}.whl"
    with WheelFile(path, "w") as archive:
        archive.write_files(root)
    return path


def launcher(runtime, target):
    common = '#!/bin/sh\nset -eu\nroot=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\n'
    if target == "linux-x64":
        common += """export LD_LIBRARY_PATH="$root/lib:$root/../brimp_webkit.libs"
export BRIMP_WEBKIT_ROOT="$(dirname -- "$root")"
export BRIMP_WEBKIT_LIBRARY="$root/lib/libwebkitgtk-6.0.so"
export WEBKIT_EXEC_PATH="$root/libexec/webkitgtk-6.0"
export WEBKIT_INJECTED_BUNDLE_PATH="$root/lib/webkitgtk-6.0/injected-bundle"
"""
    else:
        common += 'export DYLD_FRAMEWORK_PATH="$root/lib"\nexport DYLD_LIBRARY_PATH="$root/lib"\n'
    path = runtime / "brimp-webkit"
    path.write_text(common + 'exec "$root/brimp-webkit-host" "$@"\n')
    path.chmod(0o755)


def strip_macos(runtime):
    # Preserve bundle layout and entitlements; sign after removing development files.
    for path in sorted(runtime.rglob("*"), key=lambda p: len(p.parts), reverse=True):
        if path.name in {"Headers", "PrivateHeaders", "Modules"} or path.suffix in {
            ".dSYM",
            ".tbd",
        }:
            if path.is_symlink() or path.is_file():
                path.unlink()
            elif path.is_dir():
                shutil.rmtree(path)
    binaries = []
    for path in runtime.rglob("*"):
        if not path.is_file() or path.is_symlink():
            continue
        with path.open("rb") as source:
            magic = source.read(4)
        if magic in (b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe"):
            binaries.append(path)
            run("strip", "-S", "-x", path)
    for path in binaries:
        run(
            "codesign",
            "--force",
            "--sign",
            "-",
            "--preserve-metadata=entitlements,requirements,flags",
            path,
        )
    bundles = [
        p
        for p in runtime.rglob("*")
        if p.is_dir() and not p.is_symlink() and p.suffix in {".framework", ".xpc", ".app"}
    ]
    for path in sorted(bundles, key=lambda p: len(p.parts), reverse=True):
        run(
            "codesign",
            "--force",
            "--sign",
            "-",
            "--preserve-metadata=entitlements,requirements,flags",
            path,
        )


def repair_linux(stage, work):
    sys.path.insert(0, str(ROOT / "bindings"))
    from native_runtime import bundle_atomic

    # Strip before auditwheel/patchelf so ELF load alignment remains valid.
    links = {}
    for path in (stage / "runtime").rglob("*"):
        if path.is_symlink():
            links[path.relative_to(stage)] = path.readlink()
        elif path.is_file():
            with path.open("rb") as source:
                if source.read(4) == b"\x7fELF":
                    run("strip", "--strip-unneeded", path)
                    relative = os.path.relpath(stage / "runtime/lib", path.parent)
                    run("patchelf", "--set-rpath", "$ORIGIN/" + relative, path)
    bundle_atomic(stage, stage / "runtime/lib", stage / "runtime/share/licenses/gcc-runtime")
    metadata(stage, "linux_x86_64")
    raw = work / "raw"
    raw.mkdir()
    source = wheel(stage, raw, "linux_x86_64")
    repaired = work / "repaired"
    run(
        sys.executable,
        "-m",
        "auditwheel",
        "repair",
        "--plat",
        "manylinux_2_39_x86_64",
        "--only-plat",
        "-w",
        repaired,
        source,
        env={
            key: value
            for key, value in os.environ.items()
            if key not in {"LD_LIBRARY_PATH", "AUDITWHEEL_LD_LIBRARY_PATH"}
        },
    )
    (artifact,) = repaired.glob("*.whl")
    result = work / "repaired-tree"
    with zipfile.ZipFile(artifact) as archive:
        archive.extractall(result)
        for member in archive.infolist():
            if not member.is_dir():
                (result / member.filename).chmod((member.external_attr >> 16) & 0o777)
    for name, link in links.items():
        path = result / name
        target = path.parent / link
        if path.is_file() and target.is_file() and digest(path) == digest(target):
            path.unlink()
            path.symlink_to(link)
    for path in result.glob("*.dist-info"):
        shutil.rmtree(path)
    # Ship copyright notices for the libraries auditwheel grafted from Ubuntu.
    notices = result / "runtime/share/licenses/system"
    notices.mkdir(parents=True)
    # Preserve the installed distribution notices: dependencies may be reached transitively.
    for path in Path("/usr/share/doc").glob("*/copyright"):
        shutil.copy2(path, notices / f"{path.parent.name}.copyright")
    return result, "manylinux_2_39_x86_64"


def npm_metadata(name, **extra):
    return {
        "name": name,
        "version": version(),
        "description": "Bundled WebKit engine for Brimp",
        "license": "SEE LICENSE IN LICENSE",
        "repository": {"type": "git", "url": "git+https://github.com/lexiforest/brimp.git"},
        "engines": {"node": ">=22.19"},
        **extra,
    }


def pack_npm(package, output):
    run("npm", "pack", package, "--pack-destination", output, "--ignore-scripts")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--development-sdk",
        action="store_true",
        help="allow a locally packaged SDK tag; release CI must not use this",
    )
    args = parser.parse_args()
    target = {("Darwin", "arm64"): "darwin-arm64", ("Linux", "x86_64"): "linux-x64"}[
        (platform.system(), platform.machine())
    ]
    sdk = args.sdk.resolve()
    manifest = verify_sdk(sdk, target, release=not args.development_sdk)
    output = args.out.resolve()
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="brimp-webkit-package-") as directory:
        work = Path(directory)
        # Compile against a private SDK copy; rewrite development pkg-config paths only there.
        build_sdk = work / "sdk"
        shutil.copytree(sdk, build_sdk, symlinks=True)
        if target == "linux-x64":
            for path in (build_sdk / "lib/pkgconfig").glob("*.pc"):
                path.write_text(
                    path.read_text().replace(manifest["install_prefix"], str(build_sdk))
                )
        host = work / "host"
        build_environment = dict(os.environ)
        if target == "darwin-arm64":
            build_environment["MACOSX_DEPLOYMENT_TARGET"] = manifest["minimum_macos"]
        run(
            sys.executable,
            ROOT / "tools/webkit/build.py",
            "--sdk",
            build_sdk,
            "--output",
            host,
            env=build_environment,
        )
        stage = work / "stage"
        runtime = stage / "runtime"
        runtime.mkdir(parents=True)
        for name in ("lib", "libexec", "share"):
            if (sdk / name).is_dir():
                shutil.copytree(sdk / name, runtime / name, symlinks=True)
        shutil.rmtree(runtime / "lib/pkgconfig", ignore_errors=True)
        for path in runtime.rglob("*.a"):
            path.unlink()
        shutil.copy2(host / "brimp-webkit-host", runtime)
        shutil.copy2(ROOT / "LICENSE", runtime / "LICENSE")
        shutil.copy2(sdk / "manifest.json", runtime / "sdk-manifest.json")
        launcher(runtime, target)
        if target == "linux-x64":
            stage, tag = repair_linux(stage, work)
        else:
            strip_macos(runtime)
            major, minor, *_ = [int(part) for part in manifest["minimum_macos"].split(".")]
            tag = f"macosx_{major}_{minor}_arm64"
        runtime_environment = {**os.environ, "GTK_A11Y": "none", "LIBGL_ALWAYS_SOFTWARE": "1"}
        with tempfile.TemporaryDirectory(prefix="brimp-webkit-verify-") as runtime_directory:
            runtime_environment["XDG_RUNTIME_DIR"] = runtime_directory
            verify(
                stage / "runtime/brimp-webkit",
                stage,
                parser,
                runtime_environment,
                ["dbus-run-session", "--", "xvfb-run", "-a"] if target == "linux-x64" else [],
            )
        payload = output / f"brimp-webkit-{version()}-{target}.tar.xz"
        with tarfile.open(payload, "w:xz", preset=6) as archive:
            for path in sorted(stage.iterdir()):
                archive.add(path, arcname=path.name)
        provenance = {
            "version": version(),
            "target": target,
            "sha256": digest(payload),
            "sdk_tag": manifest["tag"],
            "sdk_commit": manifest["commit"],
            "development": args.development_sdk,
        }
        payload.with_suffix(".json").write_text(json.dumps(provenance, indent=2) + "\n")
        python = work / "python"
        package = python / "brimp_webkit"
        shutil.copytree(SOURCES / "python/brimp_webkit", package)
        shutil.copy2(payload, package / "runtime.tar.xz")
        (package / "runtime.json").write_text(json.dumps(provenance) + "\n")
        info = metadata(python, tag)
        (info / "entry_points.txt").write_text(
            "[console_scripts]\nbrimp-webkit = brimp_webkit:main\n"
        )
        shutil.copy2(ROOT / "LICENSE", info / "LICENSE")
        wheel(python, output, tag)
        node = work / "node"
        node.mkdir()
        os_name, arch = target.split("-")
        (node / "package.json").write_text(
            json.dumps(
                npm_metadata(
                    f"@brimp/webkit-{target}",
                    os=[os_name],
                    cpu=[arch],
                    files=["runtime.tar.xz", "runtime.json", "LICENSE"],
                    **({"libc": ["glibc"]} if target == "linux-x64" else {}),
                )
            )
        )
        shutil.copy2(payload, node / "runtime.tar.xz")
        shutil.copy2(package / "runtime.json", node)
        shutil.copy2(ROOT / "LICENSE", node)
        pack_npm(node, output)
        wrapper = work / "wrapper"
        shutil.copytree(SOURCES / "node", wrapper)
        shutil.copy2(ROOT / "LICENSE", wrapper)
        (wrapper / "package.json").write_text(
            json.dumps(
                npm_metadata(
                    "@brimp/webkit",
                    main="index.cjs",
                    types="index.d.cts",
                    files=["index.cjs", "index.d.cts", "LICENSE"],
                    optionalDependencies={
                        f"@brimp/webkit-{name}": version() for name in ["darwin-arm64", "linux-x64"]
                    },
                )
            )
        )
        pack_npm(wrapper, output)
    print(json.dumps(provenance))


if __name__ == "__main__":
    main()
