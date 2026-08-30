import hashlib
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

import package_release


class PackageReleaseTests(unittest.TestCase):
    def test_supported_targets_match_release_matrix(self):
        self.assertEqual(
            set(package_release.TARGETS),
            {
                "x86_64-unknown-linux-gnu",
                "aarch64-unknown-linux-gnu",
                "aarch64-apple-darwin",
                "x86_64-pc-windows-msvc",
            },
        )

    def test_linux_dependency_parser_rejects_missing_libraries(self):
        with mock.patch.object(
            package_release,
            "run",
            return_value="libgood.so => /native/libgood.so (0x1)\nlibbad.so => not found\n",
        ):
            with self.assertRaisesRegex(RuntimeError, "unresolved dependency"):
                package_release.linux_dependencies(Path("brimp"))

    def test_archives_have_one_top_level_directory_and_portable_checksum(self):
        with tempfile.TemporaryDirectory() as temporary:
            temporary = Path(temporary)
            package = temporary / "brimp-v1.2.3-test-target"
            package.mkdir()
            (package / "brimp").write_bytes(b"binary")
            output = temporary / "dist"

            tar_path, checksum_path = package_release.archive(
                package, output, "linux"
            )
            with tarfile.open(tar_path) as bundle:
                self.assertTrue(
                    all(
                        name == package.name or name.startswith(f"{package.name}/")
                        for name in bundle.getnames()
                    )
                )
            expected = hashlib.sha256(tar_path.read_bytes()).hexdigest()
            self.assertEqual(
                checksum_path.read_text(encoding="ascii"),
                f"{expected}  {tar_path.name}\n",
            )

            zip_path, _ = package_release.archive(package, output, "windows")
            with zipfile.ZipFile(zip_path) as bundle:
                self.assertTrue(
                    all(name.startswith(f"{package.name}/") for name in bundle.namelist())
                )


if __name__ == "__main__":
    unittest.main()
