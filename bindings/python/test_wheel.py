#!/usr/bin/env python3
"""Install and exercise a wheel without access to its build-time native SDKs."""

from __future__ import annotations

import argparse
import os
import subprocess
import tempfile
import venv
from pathlib import Path

from validate_wheel import PLATFORMS, find_wheel, project_version, validate


def environment_without_native_sdk() -> dict[str, str]:
    environment = dict(os.environ)
    runtime_dirs = set(
        environment.pop("BRIMP_NATIVE_RUNTIME_DIRS", "").split(os.pathsep)
    )
    for name in (
        "BRIMP_CURL_LIB_DIR",
        "BRIMP_JSC_LIB_DIR",
        "DYLD_LIBRARY_PATH",
        "LD_LIBRARY_PATH",
    ):
        environment.pop(name, None)
    if os.name == "nt":
        environment["PATH"] = os.pathsep.join(
            entry
            for entry in environment["PATH"].split(os.pathsep)
            if entry not in runtime_dirs
        )
    return environment


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True, choices=PLATFORMS)
    parser.add_argument("--directory", required=True, type=Path)
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    wheel = find_wheel(
        arguments.directory.resolve(), arguments.target, project_version(root)
    ).resolve()
    validate(wheel, arguments.target)

    with tempfile.TemporaryDirectory(prefix="brimp-wheel-test-") as temporary:
        temporary_path = Path(temporary)
        environment = environment_without_native_sdk()
        native = root / ".native"
        hidden_native = root / ".native-wheel-test-hidden"
        if hidden_native.exists():
            raise RuntimeError(f"stale hidden native SDK directory: {hidden_native}")
        if native.exists():
            native.rename(hidden_native)
        try:
            virtualenv = temporary_path / "venv"
            venv.EnvBuilder(with_pip=True).create(virtualenv)
            python = virtualenv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
            subprocess.run(
                [python, "-m", "pip", "install", "--no-index", wheel],
                check=True,
                env=environment,
            )
            subprocess.run(
                [python, "-m", "pip", "install", "pytest>=8,<10"],
                check=True,
                env=environment,
            )
            subprocess.run(
                [python, "-m", "pytest", root / "bindings/python/test_api.py"],
                check=True,
                cwd=temporary_path,
                env=environment,
            )
        finally:
            if hidden_native.exists():
                hidden_native.rename(native)


if __name__ == "__main__":
    main()
