"""Pinned SDK release wiring and license collection."""

from pathlib import Path
import pytest
import prepare_native as sdk


@pytest.fixture
def prepared(tmp_path, monkeypatch):
    monkeypatch.delenv("GITHUB_PATH", raising=False)
    monkeypatch.setattr(sdk, "download", lambda asset, cache: tmp_path / asset.filename)

    def extract(archive, destination):
        destination.mkdir(parents=True, exist_ok=True)
        windows = ".x86_64-win32." in Path(archive).name
        root = destination if "curl" in Path(archive).name else destination / "sdk"
        paths = [
            "lib/JavaScriptCore.framework/JavaScriptCore",
            "lib/libJavaScriptCore.so",
            "lib/libicudata.so.77",
            "lib/libicui18n.so.77",
            "lib/libicuuc.so.77",
            "lib/JavaScriptCore.lib",
            "bin/JavaScriptCore.dll",
            "bin/icudt77.dll",
            "bin/icuin77.dll",
            "bin/icuuc77.dll",
            "share/licenses/JavaScriptCore/COPYING.LIB",
            "share/licenses/icu/LICENSE",
        ]
        if "curl" in Path(archive).name:
            paths = ["include/curl-impersonate/webcrypto.h", "LICENSE"]
            paths += (
                ["lib/libcurl-impersonate_imp.lib", "lib/libcurl-impersonate.dll"]
                if windows
                else ["libcurl-impersonate.so", "libcurl-impersonate.dylib"]
            )
        for name in paths:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.touch()

    monkeypatch.setattr(sdk, "extract", extract)
    return tmp_path


@pytest.mark.parametrize("target", sdk.TARGETS)
def test_all_release_assets_are_pinned(target):
    assert len(sdk.TARGETS[target].jsc.sha256) == 64
    assert len(sdk.TARGETS[target].curl.sha256) == 64


@pytest.mark.parametrize("target", sdk.TARGETS)
def test_standalone_jsc_does_not_select_curl(prepared, target):
    values = sdk.prepare(target, prepared / "output")
    assert "BRIMP_CURL_LIB_DIR" not in values
    assert Path(values["BRIMP_JSC_LIB_DIR"]).is_dir()
    assert Path(values["BRIMP_LICENSE_DIR"]).is_dir()
    assert not (Path(values["BRIMP_LICENSE_DIR"]) / "curl-sdk").exists()


@pytest.mark.parametrize("target", sdk.TARGETS)
def test_pinned_curl_release_is_normalized(prepared, target):
    values = sdk.prepare(target, prepared / "output", curl_release=True)
    assert Path(values["BRIMP_CURL_LIB_DIR"]).is_dir()
    assert Path(values["BRIMP_CURL_SDK"], "include/curl-impersonate/webcrypto.h").is_file()
    assert Path(values["BRIMP_LICENSE_DIR"], "curl-sdk/LICENSE").is_file()
    assert Path(values["BRIMP_LICENSE_DIR"], "jsc-sdk/icu/LICENSE").is_file()
    assert Path(values["BRIMP_LICENSE_DIR"], "skia-LICENSE").is_file()


@pytest.mark.parametrize("target", sdk.TARGETS)
def test_explicit_curl_sdk_still_requires_crypto_header(prepared, target):
    prefix = prepared / "curl-sdk"
    lib = prefix / "lib"
    lib.mkdir(parents=True)
    for name in [
        "libcurl-impersonate.dylib",
        "libcurl-impersonate.so",
        "libcurl-impersonate_imp.lib",
        "libcurl-impersonate.dll",
    ]:
        (lib / name).touch()
    (prefix / "LICENSE").touch()
    with pytest.raises(RuntimeError, match="webcrypto.h"):
        sdk.prepare(target, prepared / "output", prefix)
    header = prefix / "include/curl-impersonate/webcrypto.h"
    header.parent.mkdir(parents=True)
    header.touch()
    values = sdk.prepare(target, prepared / "output", prefix)
    assert values["BRIMP_CURL_LIB_DIR"] == str(lib.resolve())
    assert str(lib.resolve()) in values["BRIMP_NATIVE_RUNTIME_DIRS"]


def test_curl_sdk_and_release_are_mutually_exclusive(prepared):
    with pytest.raises(RuntimeError, match="either"):
        sdk.prepare(
            "x86_64-unknown-linux-gnu",
            prepared / "output",
            prepared / "curl-sdk",
            curl_release=True,
        )
