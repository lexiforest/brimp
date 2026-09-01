import tempfile
import zipfile
from pathlib import Path

import prepare_native
import pytest
import validate_wheel

TEST_VERSION = "1.2.3"


def test_unpinned_digest_fails_before_download():
    asset = prepare_native.Asset("test.tar.gz", "PENDING", "https://invalid")
    with tempfile.TemporaryDirectory() as temporary:
        with pytest.raises(RuntimeError, match="does not have a pinned SHA-256"):
            prepare_native.download(asset, Path(temporary))


def test_all_release_digests_are_pinned():
    for target in prepare_native.TARGETS.values():
        assert len(target.jsc.sha256) == 64
        assert len(target.curl.sha256) == 64


def test_defuddle_notices_are_available_to_wheel_packaging():
    root = Path(__file__).resolve().parents[2]
    with tempfile.TemporaryDirectory() as temporary:
        destination = Path(temporary) / "defuddle"
        prepare_native.copy_defuddle_licenses(root, destination)
        assert (destination / "NOTICE.md").is_file()
        assert (destination / "licenses/defuddle-LICENSE").is_file()


def test_expected_wheel_name_is_exact():
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        wheel = directory / f"brimp-{TEST_VERSION}-cp310-abi3-manylinux_2_28_x86_64.whl"
        wheel.touch()
        assert validate_wheel.find_wheel(
            directory, "x86_64-unknown-linux-gnu", TEST_VERSION
        ) == wheel


def test_complete_synthetic_wheels_validate():
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        for target, platform in validate_wheel.PLATFORMS.items():
            wheel = directory / f"brimp-{TEST_VERSION}-cp310-abi3-{platform}.whl"
            extension = "brimp/_brimp.pyd" if "windows" in target else "brimp/_brimp.so"
            library_suffix = ".dll" if "windows" in target else ".so"
            if "apple" in target:
                library_suffix = ""
            names = [
                extension,
                *(
                    f"{validate_wheel.NATIVE_LIBRARY_PREFIX[target]}"
                    f"{library}-deadbeef{library_suffix}"
                    for library in validate_wheel.REQUIRED_LIBRARIES[target]
                ),
                *validate_wheel.REQUIRED_LICENSES,
            ]
            if "linux-gnu" in target:
                names.append("brimp/licenses/jsc-sdk/icu/LICENSE")
            with zipfile.ZipFile(wheel, "w") as archive:
                for name in names:
                    archive.writestr(name, "test")
            validate_wheel.validate(wheel, target)
