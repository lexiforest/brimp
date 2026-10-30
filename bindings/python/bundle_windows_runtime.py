#!/usr/bin/env python3
"""Add the Visual C++ runtime DLLs used by the Windows JSC SDK to a repaired wheel."""

from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import os
import shutil
import tempfile
import zipfile
from pathlib import Path


RUNTIME_DLLS = ("msvcp140.dll", "msvcp140_2.dll", "vcruntime140.dll", "vcruntime140_1.dll")


def record(root: Path) -> None:
    record_path = next(root.glob("*.dist-info/RECORD"))
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path != record_path:
            data = path.read_bytes()
            digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
            rows.append([path.relative_to(root).as_posix(), "sha256=" + digest, str(len(data))])
    rows.append([record_path.relative_to(root).as_posix(), "", ""])
    buffer = io.StringIO()
    csv.writer(buffer, lineterminator="\n").writerows(rows)
    record_path.write_text(buffer.getvalue())


def archive(root: Path, output: Path) -> None:
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as wheel:
        for path in sorted(root.rglob("*")):
            if path.is_file():
                info = zipfile.ZipInfo(path.relative_to(root).as_posix(), (1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = (0o100755 if path.suffix in (".so", ".pyd", ".dll") else 0o100644) << 16
                wheel.writestr(info, path.read_bytes())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wheel", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--runtime-dir", type=Path)
    arguments = parser.parse_args()
    runtime = arguments.runtime_dir or Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32"
    arguments.out.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix="brimp-wheel-runtime-"))
    try:
        with zipfile.ZipFile(arguments.wheel) as wheel:
            wheel.extractall(temporary)
        libraries = temporary / "brimp.libs"
        if not libraries.is_dir():
            raise RuntimeError(f"repaired wheel has no brimp.libs directory: {arguments.wheel}")
        for name in RUNTIME_DLLS:
            source = runtime / name
            if not source.is_file():
                raise RuntimeError(f"Visual C++ runtime DLL is missing: {source}")
            target = libraries / name
            if not target.exists():
                shutil.copyfile(source, target)
        record(temporary)
        output = arguments.out / arguments.wheel.name
        archive(temporary, output)
        print(output)
    finally:
        shutil.rmtree(temporary, ignore_errors=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
