"""Bundle libatomic into a repaired Linux wheel, preserving its tag and RECORD."""

import argparse
import sys
import tempfile
import zipfile
from pathlib import Path

from wheel.wheelfile import WheelFile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from native_runtime import bundle_atomic


def bundle(wheel: Path, output: Path) -> Path:
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="brimp-linux-wheel-") as directory:
        root = Path(directory)
        with zipfile.ZipFile(wheel) as archive:
            archive.extractall(root)
            for member in archive.infolist():
                if not member.is_dir():
                    (root / member.filename).chmod((member.external_attr >> 16) & 0o777)
        bundle_atomic(root, root / "brimp.libs", root / "brimp/bundled/licenses/native/gcc-runtime")
        result = output / wheel.name
        with WheelFile(result, "w") as archive:
            archive.write_files(root)
        return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wheel", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    print(bundle(args.wheel, args.out))
