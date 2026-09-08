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


def test_macos_bundles_native_dependencies_with_worker_only(tmp_path, monkeypatch):
    worker = tmp_path / "custom-worker-build-name"
    worker.write_bytes(b"worker")
    package = tmp_path / "package"
    package.mkdir()
    cli = package / "brimp"
    cli.write_bytes(b"runtime-free-cli")
    jsc = tmp_path / "jsc"
    (jsc / "JavaScriptCore.framework").mkdir(parents=True)
    curl = tmp_path / "curl"
    curl.mkdir()
    (curl / "libcurl-impersonate.dylib").write_bytes(b"curl")
    commands = []
    monkeypatch.setattr(package_release, "otool_dependencies", lambda path: [str(jsc / "JavaScriptCore.framework/JavaScriptCore"), str(curl / "libcurl-impersonate.dylib")])
    monkeypatch.setattr(package_release, "run", lambda *args, **kwargs: commands.append(args) or "")
    package_release.package_macos(worker, package, jsc, curl)
    assert cli.read_bytes() == b"runtime-free-cli"
    assert (package / "lite-worker").read_bytes() == b"worker"
    assert all(command[-1] == str(package / "lite-worker") for command in commands)


def test_validation_probes_worker_and_rejects_native_cli_linkage(tmp_path, monkeypatch):
    import pytest
    (tmp_path / "brimp").write_bytes(b"cli")
    (tmp_path / "lite-worker").write_bytes(b"worker")
    commands = []
    monkeypatch.setattr(package_release, "run", lambda *args, **kwargs: commands.append(args) or "")
    monkeypatch.setattr(package_release, "otool_dependencies", lambda path: ["/usr/lib/libSystem.B.dylib"])
    package_release.validate(tmp_path, "macos", [])
    assert commands[-1] == (str(tmp_path / "brimp"), "doctor", "--worker-path", str(tmp_path / "lite-worker"))
    monkeypatch.setattr(package_release, "otool_dependencies", lambda path: ["@rpath/JavaScriptCore.framework/JavaScriptCore"])
    with pytest.raises(RuntimeError, match="must not link browser"):
        package_release.validate(tmp_path, "macos", [])
