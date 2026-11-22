"""Exercise libatomic bundling with real ELF dependencies and relocated loads."""

import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

import pytest
from wheel.wheelfile import WheelFile

sys.path.insert(0, str(Path(__file__).parent))
sys.path.insert(0, str(Path(__file__).parent / "python"))
from bundle_linux_runtime import bundle
from native_runtime import bundle_linux

pytestmark = pytest.mark.skipif(sys.platform != "linux", reason="ELF runtime tests")


@pytest.fixture
def consumer(tmp_path):
    for tool in ("cc", "patchelf", "strip"):
        assert shutil.which(tool), f"Install {tool} to run Linux packaging tests"
    source = tmp_path / "atomic.c"
    source.write_text(
        "#include <stdatomic.h>\n"
        "_Atomic(__int128) value = 42;\n"
        "int answer(void) { return atomic_load(&value); }\n"
    )
    library = tmp_path / "consumer.so"
    subprocess.run(["cc", "-shared", "-fPIC", source, "-latomic", "-o", library], check=True)
    return library


def check_loaded_atomic(consumer, bundled):
    environment = {key: value for key, value in os.environ.items() if key != "LD_LIBRARY_PATH"}
    script = (
        "import ctypes,sys; from pathlib import Path; "
        "assert ctypes.CDLL(sys.argv[1]).answer() == 42; "
        "maps=Path('/proc/self/maps').read_text(); "
        "assert str(Path(sys.argv[2]).resolve()) in maps, maps"
    )
    subprocess.run([sys.executable, "-c", script, consumer, bundled], check=True, env=environment)


def test_relocated_node_cli_runtime_includes_atomic_and_notices(tmp_path, consumer):
    jsc, curl, package = [tmp_path / name for name in ("jsc", "curl", "package")]
    for sdk in (jsc, curl):
        (sdk / "lib").mkdir(parents=True)
    package.mkdir()
    bundle_linux(consumer, jsc, curl, package)
    moved = tmp_path / "relocated"
    package.rename(moved)
    consumer.unlink()
    check_loaded_atomic(moved / "consumer.so", moved / "native/libatomic.so.1")
    assert (moved / "native/licenses/gcc-runtime/COPYING.RUNTIME").is_file()


def test_repaired_wheel_includes_atomic_with_valid_record(tmp_path, consumer):
    stage = tmp_path / "stage"
    (stage / "brimp.libs").mkdir(parents=True)
    shutil.copy2(consumer, stage / "brimp.libs/consumer.so")
    info = stage / "brimp-1.2.3.dist-info"
    info.mkdir()
    (info / "METADATA").write_text("Metadata-Version: 2.4\nName: brimp\nVersion: 1.2.3\n")
    (info / "WHEEL").write_text("Wheel-Version: 1.0\nRoot-Is-Purelib: false\n")
    original = tmp_path / f"brimp-1.2.3-cp311-abi3-linux_{platform.machine()}.whl"
    with WheelFile(original, "w") as archive:
        archive.write_files(stage)
    result = bundle(original, tmp_path / "output")
    moved = tmp_path / "installed elsewhere"
    with WheelFile(result) as archive:
        # WheelFile validates RECORD while reading every member.
        for name in archive.namelist():
            archive.read(name)
        archive.extractall(moved)
    shutil.rmtree(stage)
    consumer.unlink()
    check_loaded_atomic(moved / "brimp.libs/consumer.so", moved / "brimp.libs/libatomic.so.1")
    assert (moved / "brimp/bundled/licenses/native/gcc-runtime/COPYING3").is_file()
