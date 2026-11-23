"""Smoke-test the base Node payload before the companion is built.

This deliberately checks archive contents without resolving dependencies. The
WebKit release matrix separately installs packages with their dependencies and
exercises both engines, including automatic companion discovery.
"""

import argparse
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

from package_node import ROOT, validate


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", required=True, type=Path)
    args = parser.parse_args()
    validate(args.directory)
    with tempfile.TemporaryDirectory(prefix="brimp-node-payload-") as directory:
        root = Path(directory)
        for path in args.directory.glob("*.tgz"):
            with tarfile.open(path) as archive:
                name = json.load(archive.extractfile("package/package.json"))["name"]
                archive.extractall(root / "unpacked", filter="data")
            destination = root / "node_modules" / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            (root / "unpacked/package").rename(destination)
        shutil.copy2(ROOT / "bindings/node/smoke.mjs", root / "smoke.mjs")
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("BRIMP_", "DYLD_", "__XPC_DYLD_"))
            and key not in {"LD_LIBRARY_PATH", "NODE_PATH"}
        }
        subprocess.run(["node", "smoke.mjs"], cwd=root, env=environment, check=True, timeout=120)


if __name__ == "__main__":
    main()
