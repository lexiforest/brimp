"""SDK wiring must not silently select curl releases without WebCrypto exports."""
from pathlib import Path
import pytest
import prepare_native as sdk


@pytest.fixture
def prepared_jsc(tmp_path, monkeypatch):
    monkeypatch.delenv('GITHUB_PATH', raising=False)
    monkeypatch.setattr(sdk, 'download', lambda asset, cache: tmp_path / asset.filename)

    def extract(archive, destination):
        root = destination / 'sdk'
        for name in ['lib/JavaScriptCore.framework/JavaScriptCore',
                     'lib/libJavaScriptCore.so', 'lib/libicudata.so.77',
                     'lib/libicui18n.so.77', 'lib/libicuuc.so.77',
                     'lib/JavaScriptCore.lib', 'bin/JavaScriptCore.dll',
                     'bin/icudt77.dll', 'bin/icuin77.dll', 'bin/icuuc77.dll',
                     'share/licenses/LICENSE']:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.touch()
    monkeypatch.setattr(sdk, 'extract', extract)
    return tmp_path


@pytest.mark.parametrize('target', sdk.TARGETS)
def test_standalone_jsc_does_not_select_a_curl_sdk(prepared_jsc, target):
    values = sdk.prepare(target, prepared_jsc / 'output')
    assert 'BRIMP_CURL_LIB_DIR' not in values
    assert Path(values['BRIMP_JSC_LIB_DIR']).is_dir()


@pytest.mark.parametrize('target', sdk.TARGETS)
def test_runtime_uses_explicit_sdk_and_requires_crypto_header(prepared_jsc, target):
    prefix = prepared_jsc / 'curl-sdk'
    lib = prefix / 'lib'
    lib.mkdir(parents=True)
    for name in ['libcurl-impersonate.dylib', 'libcurl-impersonate.so',
                 'libcurl-impersonate_imp.lib', 'libcurl-impersonate.dll']:
        (lib / name).touch()
    with pytest.raises(RuntimeError, match='webcrypto.h'):
        sdk.prepare(target, prepared_jsc / 'output', prefix)
    header = prefix / 'include/curl-impersonate/webcrypto.h'
    header.parent.mkdir(parents=True)
    header.touch()
    values = sdk.prepare(target, prepared_jsc / 'output', prefix)
    assert values['BRIMP_CURL_LIB_DIR'] == str(lib.resolve())
    assert str(lib.resolve()) in values['BRIMP_NATIVE_RUNTIME_DIRS']
