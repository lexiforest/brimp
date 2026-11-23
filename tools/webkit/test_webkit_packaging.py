"""Release integrity and install-time relocation regression tests."""

import argparse
import hashlib
import importlib.util
import io
import json
import shutil
import struct
import subprocess
import sys
import tarfile
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent))
from build import verify
from download import digest, verify_sdk
from package import ROOT


@pytest.mark.parametrize("outside_sdk", [False, True])
def test_helper_identity_resolves_sdk_symlinks(tmp_path, monkeypatch, outside_sdk):
    sdk = tmp_path / "sdk"
    sdk.mkdir()
    alias = tmp_path / "sdk-alias"
    alias.symlink_to(sdk, target_is_directory=True)
    framework = (tmp_path if outside_sdk else sdk) / "WebKit.framework"
    framework.mkdir()
    payload = json.dumps({"id": 1, "result": {"protocol": 1, "framework": str(framework)}}).encode()
    monkeypatch.setattr(
        subprocess,
        "run",
        lambda *args, **kwargs: subprocess.CompletedProcess(
            args, 0, struct.pack(">I", len(payload)) + payload, b""
        ),
    )
    if outside_sdk:
        with pytest.raises(SystemExit):
            verify(tmp_path / "host", alias, argparse.ArgumentParser())
    else:
        verify(tmp_path / "host", alias, argparse.ArgumentParser())


def load_runtime(path):
    spec = importlib.util.spec_from_file_location("test_webkit_runtime", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@pytest.fixture
def runtime(tmp_path, monkeypatch):
    package = tmp_path / "installed/brimp_webkit"
    shutil.copytree(ROOT / "bindings/webkit/python/brimp_webkit", package)
    archive = package / "runtime.tar.xz"
    with tarfile.open(archive, "w:gz") as bundle:
        member = tarfile.TarInfo("runtime/brimp-webkit")
        data = b"#!/bin/sh\necho packaged-engine\n"
        member.size = len(data)
        member.mode = 0o755
        bundle.addfile(member, io.BytesIO(data))
        link = tarfile.TarInfo("runtime/link")
        link.type = tarfile.SYMTYPE
        link.linkname = "brimp-webkit"
        bundle.addfile(link)
    (package / "runtime.json").write_text(json.dumps({"sha256": digest(archive)}))
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "cache with spaces"))
    return package


def test_installed_runtime_preserves_symlinks_and_execution(runtime):
    module = load_runtime(runtime / "__init__.py")
    executable = Path(module.executable_path())
    assert executable.parent.joinpath("link").is_symlink()
    assert subprocess.check_output([executable], text=True) == "packaged-engine\n"
    assert module.executable_path() == str(executable)


def test_corrupt_payload_is_rejected_before_extraction(runtime):
    (runtime / "runtime.tar.xz").write_bytes(b"corrupt")
    with pytest.raises(RuntimeError, match="checksum"):
        load_runtime(runtime / "__init__.py").executable_path()


def test_path_traversal_is_rejected(runtime, tmp_path):
    archive = runtime / "runtime.tar.xz"
    with tarfile.open(archive, "w:gz") as bundle:
        member = tarfile.TarInfo("../../escaped")
        member.size = 1
        bundle.addfile(member, io.BytesIO(b"x"))
    (runtime / "runtime.json").write_text(json.dumps({"sha256": digest(archive)}))
    with pytest.raises(tarfile.FilterError):
        load_runtime(runtime / "__init__.py").executable_path()
    assert not (tmp_path / "escaped").exists()


def test_node_installed_package_extracts_without_network(runtime, tmp_path):
    if not shutil.which("node"):
        pytest.skip("Node.js is not installed")
    modules = tmp_path / "node_modules/@brimp"
    wrapper = modules / "webkit"
    shutil.copytree(ROOT / "bindings/webkit/node", wrapper)
    (wrapper / "package.json").write_text('{"main":"index.cjs"}')
    target = subprocess.check_output(
        ["node", "-p", "process.platform+'-'+process.arch"], text=True
    ).strip()
    package = modules / f"webkit-{target}"
    shutil.copytree(runtime, package)
    result = subprocess.check_output(
        [
            "node",
            "-e",
            "const p=require('@brimp/webkit').executablePath(); console.log(require('node:child_process').execFileSync(p,{encoding:'utf8'}).trim());",
        ],
        cwd=tmp_path,
        text=True,
    )
    assert result.strip() == "packaged-engine"


def test_sdk_requires_relocation_and_integrity(tmp_path):
    sdk = tmp_path / "sdk"
    sdk.mkdir()
    (sdk / "library").write_bytes(b"engine")
    metadata = {
        "format": 1,
        "tag": "local-test",
        "target": "x86_64-linux-gnu",
        "commit": "a" * 40,
        "files": {"library": {"sha256": hashlib.sha256(b"engine").hexdigest()}},
    }
    manifest = sdk / "manifest.json"
    manifest.write_text(json.dumps(metadata))
    with pytest.raises(RuntimeError, match="relocatable"):
        verify_sdk(sdk, "linux-x64", release=False)
    metadata["runtime_layout_version"] = 1
    manifest.write_text(json.dumps(metadata))
    assert verify_sdk(sdk, "linux-x64", release=False)["commit"] == "a" * 40
    with pytest.raises(RuntimeError, match="tag"):
        verify_sdk(sdk, "linux-x64")
    (sdk / "library").write_bytes(b"changed")
    with pytest.raises(RuntimeError, match="hashes"):
        verify_sdk(sdk, "linux-x64", release=False)


@pytest.fixture
def companion_release(tmp_path):
    import zipfile

    from download import CONFIG
    from package import version

    tag = json.loads(CONFIG.read_text())["tag"]
    payload = b"test engine payload"
    expected = hashlib.sha256(payload).hexdigest()
    artifacts = {}
    for target, wheel_tag in [
        ("darwin-arm64", "macosx_26_0_arm64"),
        ("linux-x64", "manylinux_2_39_x86_64"),
    ]:
        info = {
            "version": version(),
            "target": target,
            "sha256": expected,
            "sdk_tag": tag,
            "sdk_commit": "a" * 40,
            "development": False,
        }
        wheel = tmp_path / f"brimp_webkit-{version()}-py3-none-{wheel_tag}.whl"
        with zipfile.ZipFile(wheel, "w") as archive:
            archive.writestr("brimp_webkit/runtime.json", json.dumps(info))
            archive.writestr("brimp_webkit/runtime.tar.xz", payload)
        standalone = tmp_path / f"brimp-webkit-{version()}-{target}.tar.xz"
        standalone.write_bytes(payload)
        standalone.with_suffix(".json").write_text(json.dumps(info))
        artifacts[target] = info
        with tarfile.open(tmp_path / f"brimp-webkit-{target}-{version()}.tgz", "w:gz") as archive:
            for name, data in {
                "package.json": json.dumps(
                    {"name": f"@brimp/webkit-{target}", "version": version()}
                ).encode(),
                "runtime.json": json.dumps(info).encode(),
                "runtime.tar.xz": payload,
            }.items():
                member = tarfile.TarInfo(f"package/{name}")
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
    with tarfile.open(tmp_path / f"brimp-webkit-{version()}.tgz", "w:gz") as archive:
        data = json.dumps(
            {
                "name": "@brimp/webkit",
                "version": version(),
                "optionalDependencies": {
                    f"@brimp/webkit-{target}": version() for target in artifacts
                },
            }
        ).encode()
        member = tarfile.TarInfo("package/package.json")
        member.size = len(data)
        archive.addfile(member, io.BytesIO(data))
    return tmp_path, artifacts


def test_release_gate_allows_publication_without_windows(companion_release):
    from validate import validate

    directory, _ = companion_release
    validate(directory)


@pytest.mark.parametrize("wheel_tag", ["macosx_26_0_arm64", "manylinux_2_39_x86_64"])
def test_release_gate_requires_supported_platforms(companion_release, wheel_tag):
    from validate import validate

    directory, _ = companion_release
    next(directory.glob(f"*-{wheel_tag}.whl")).unlink()
    with pytest.raises(RuntimeError, match="exactly two WebKit wheels"):
        validate(directory)


def test_release_gate_checks_provenance_and_identical_payloads(companion_release):
    import zipfile

    from package import version
    from validate import validate

    tmp_path, artifacts = companion_release
    wheel = tmp_path / f"brimp_webkit-{version()}-py3-none-manylinux_2_39_x86_64.whl"
    with zipfile.ZipFile(wheel) as archive:
        payload = archive.read("brimp_webkit/runtime.tar.xz")
    artifacts["linux-x64"]["development"] = True
    with zipfile.ZipFile(wheel, "w") as archive:
        archive.writestr("brimp_webkit/runtime.json", json.dumps(artifacts["linux-x64"]))
        archive.writestr("brimp_webkit/runtime.tar.xz", payload)
    with pytest.raises(RuntimeError, match="provenance"):
        validate(tmp_path)


def test_companion_metadata_uses_its_own_version(tmp_path, monkeypatch):
    import package

    config = tmp_path / "release.json"
    config.write_text(json.dumps({"version": "7.8.9"}))
    monkeypatch.setattr(package, "CONFIG", config)
    assert package.version() == "7.8.9"
    assert package.npm_metadata("@brimp/webkit")["version"] == "7.8.9"
    metadata = package.metadata(tmp_path, "linux_x86_64")
    assert metadata.name == "brimp_webkit-7.8.9.dist-info"
    assert "Version: 7.8.9\n" in (metadata / "METADATA").read_text()


@pytest.mark.parametrize(
    "system,machine,target", [("Darwin", "arm64", "darwin-arm64"), ("Linux", "x86_64", "linux-x64")]
)
def test_published_companion_is_pinned_for_both_registries(monkeypatch, system, machine, target):
    import test_distribution

    monkeypatch.setattr(test_distribution.platform, "system", lambda: system)
    monkeypatch.setattr(test_distribution.platform, "machine", lambda: machine)
    assert test_distribution.companion_sources(None, "3.4.5") == (
        ["brimp-webkit==3.4.5"],
        ["@brimp/webkit@3.4.5", f"@brimp/webkit-{target}@3.4.5"],
    )


@pytest.mark.parametrize("minimum", ["15.0", "26.0", "27.0"])
def test_macos_sdk_requires_requested_deployment_target(tmp_path, minimum):
    from download import CONFIG

    config = json.loads(CONFIG.read_text())
    sdk = tmp_path / "sdk"
    sdk.mkdir()
    (sdk / "manifest.json").write_text(
        json.dumps(
            {
                "format": 1,
                "tag": config["tag"],
                "target": "arm64-macos",
                "commit": "a" * 40,
                "minimum_macos": minimum,
                "files": {},
            }
        )
    )
    if minimum == "26.0":
        assert verify_sdk(sdk, "darwin-arm64")["minimum_macos"] == "26.0"
    else:
        with pytest.raises(RuntimeError, match="rebuilt for macOS 26.0"):
            verify_sdk(sdk, "darwin-arm64")
