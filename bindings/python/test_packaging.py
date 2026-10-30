import tempfile
import unittest
import zipfile
from pathlib import Path

import validate_wheel

TEST_VERSION = "1.2.3"


class PackagingTests(unittest.TestCase):
    def test_expected_wheel_name_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            wheel = directory / f"brimp-{TEST_VERSION}-cp311-abi3-manylinux_2_34_x86_64.whl"
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
                wheel = directory / f"brimp-{TEST_VERSION}-cp311-abi3-{platform}.whl"
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
                    names.append("brimp/bundled/licenses/native/jsc-sdk/icu/LICENSE")
                with zipfile.ZipFile(wheel, "w") as archive:
                    for name in names:
                        archive.writestr(name, "test")
                validate_wheel.validate(wheel, target)


if __name__ == "__main__":
    unittest.main()
