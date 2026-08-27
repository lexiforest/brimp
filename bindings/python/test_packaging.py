import tempfile
import unittest
import zipfile
from pathlib import Path

import prepare_native
import validate_wheel

TEST_VERSION = "1.2.3"


class PackagingTests(unittest.TestCase):
    def test_unpinned_digest_fails_before_download(self):
        asset = prepare_native.Asset("test.tar.gz", "PENDING", "https://invalid")
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(RuntimeError, "does not have a pinned SHA-256"):
                prepare_native.download(asset, Path(temporary))

    def test_all_release_digests_are_pinned(self):
        for target in prepare_native.TARGETS.values():
            self.assertEqual(len(target.jsc.sha256), 64)
            self.assertEqual(len(target.curl.sha256), 64)

    def test_expected_wheel_name_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            wheel = directory / f"brimp-{TEST_VERSION}-cp310-abi3-manylinux_2_28_x86_64.whl"
            wheel.touch()
            self.assertEqual(
                validate_wheel.find_wheel(
                    directory, "x86_64-unknown-linux-gnu", TEST_VERSION
                ),
                wheel,
            )

    def test_complete_synthetic_wheels_validate(self):
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


if __name__ == "__main__":
    unittest.main()
