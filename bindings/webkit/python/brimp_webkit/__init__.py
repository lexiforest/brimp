"""Locate the engine bundled with the brimp-webkit distribution."""

import errno
import hashlib
import json
import os
import shutil
import sys
import tarfile
import tempfile
from pathlib import Path


def executable_path() -> str:
    """Extract the bundled runtime atomically; no network access is performed."""
    package = Path(__file__).parent
    metadata = json.loads((package / "runtime.json").read_text())
    archive = package / "runtime.tar.xz"
    cache = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "brimp/webkit"
    destination = cache / metadata["sha256"]
    executable = destination / "runtime/brimp-webkit"
    if executable.is_file():
        return str(executable)
    with archive.open("rb") as source:
        if hashlib.file_digest(source, "sha256").hexdigest() != metadata["sha256"]:
            raise RuntimeError("The bundled WebKit runtime checksum does not match")
    cache.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix="extract-", dir=cache))
    try:
        with tarfile.open(archive) as bundle:
            bundle.extractall(temporary, filter="data")
        if not (temporary / "runtime/brimp-webkit").is_file():
            raise RuntimeError("The WebKit runtime has no launcher")
        try:
            temporary.rename(destination)
        except OSError as error:
            if error.errno not in (errno.EEXIST, errno.ENOTEMPTY) or not executable.is_file():
                raise
    finally:
        shutil.rmtree(temporary, ignore_errors=True)
    return str(executable)


def main():
    os.execv(executable_path(), ["brimp-webkit", *sys.argv[1:]])
