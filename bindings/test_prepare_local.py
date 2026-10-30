"""Development bindings must load their native dependencies on every platform."""

import os

import pytest

import prepare_local


@pytest.mark.parametrize(
    "platform,library,extension",
    [
        ("darwin", "lib_brimp.dylib", "_brimp.abi3.so"),
        ("linux", "lib_brimp.so", "_brimp.abi3.so"),
        ("win32", "_brimp.dll", "_brimp.pyd"),
    ],
)
def test_prepare_python_binding(tmp_path, monkeypatch, platform, library, extension):
    monkeypatch.setattr(prepare_local.sys, "platform", platform)
    build = tmp_path / "target/debug"
    build.mkdir(parents=True)
    (build / library).write_bytes(b"new extension")
    destination = tmp_path / "bindings/python/python/brimp"
    destination.mkdir(parents=True)
    (destination / extension).write_bytes(b"old extension")
    sdk = tmp_path / "sdk"
    sdk.mkdir()
    for name in ("JavaScriptCore.dll", "libcurl-impersonate.dll", "icuuc77.dll"):
        (sdk / name).write_bytes(name.encode())
    monkeypatch.setenv("BRIMP_NATIVE_RUNTIME_DIRS", str(sdk))

    prepare_local.prepare(tmp_path, only="python")

    assert (destination / extension).read_bytes() == b"new extension"
    assert not (tmp_path / "bindings/node/brimp.node").exists()
    if platform == "win32":
        for dll in sdk.glob("*.dll"):
            assert (destination / dll.name).read_bytes() == dll.read_bytes()
    else:
        assert not list(destination.glob("*.dll"))


def test_windows_requires_complete_sdk(tmp_path, monkeypatch):
    monkeypatch.setattr(prepare_local.sys, "platform", "win32")
    monkeypatch.setenv("BRIMP_NATIVE_RUNTIME_DIRS", str(tmp_path))
    with pytest.raises(RuntimeError, match="JSC and curl DLLs"):
        prepare_local.prepare(tmp_path)


def test_windows_release_node_copies_dlls_from_both_sdks(tmp_path, monkeypatch):
    monkeypatch.setattr(prepare_local.sys, "platform", "win32")
    build = tmp_path / "target/release"
    build.mkdir(parents=True)
    (build / "brimp_node.dll").write_bytes(b"node extension")
    destination = tmp_path / "bindings/node"
    destination.mkdir(parents=True)
    directories = [tmp_path / "jsc/bin", tmp_path / "curl/lib"]
    for directory, name in zip(directories, ["JavaScriptCore.dll", "libcurl-impersonate.dll"]):
        directory.mkdir(parents=True)
        (directory / name).write_bytes(b"dependency")
    monkeypatch.setenv("BRIMP_NATIVE_RUNTIME_DIRS", os.pathsep.join(map(str, directories)))

    prepare_local.prepare(tmp_path, release=True, only="node")

    assert (destination / "brimp.node").read_bytes() == b"node extension"
    assert sorted(p.name for p in destination.glob("*.dll")) == [
        "JavaScriptCore.dll",
        "libcurl-impersonate.dll",
    ]
