"""Exercise an unpacked CLI distribution with no access to the build SDKs."""

import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import os
from pathlib import Path
import subprocess
import tarfile
import threading
import zipfile

import pytest

ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture(scope="module")
def installed_cli(tmp_path_factory):
    archive = os.environ.get("BRIMP_CLI_ARCHIVE")
    if not archive:
        pytest.skip("set BRIMP_CLI_ARCHIVE to test an assembled distribution")
    archive = Path(archive).resolve()
    destination = tmp_path_factory.mktemp("standalone-cli")
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as package:
            package.extractall(destination)
    else:
        with tarfile.open(archive, "r:gz") as package:
            package.extractall(destination, filter="data")
    directory = next(destination.iterdir())
    executable = directory / ("brimp.exe" if os.name == "nt" else "brimp")
    native, hidden = ROOT / ".native", ROOT / ".native-cli-test-hidden"
    assert not hidden.exists(), f"stale hidden SDK directory: {hidden}"
    if native.exists():
        native.rename(hidden)
    try:
        yield executable
    finally:
        if hidden.exists():
            hidden.rename(native)


def invoke(installed_cli, *arguments, source=None):
    executable = installed_cli
    environment = dict(os.environ)
    runtime_dirs = set(environment.get("BRIMP_NATIVE_RUNTIME_DIRS", "").split(os.pathsep))
    for name in list(environment):
        if name.startswith("BRIMP_") or name in (
            "DYLD_LIBRARY_PATH",
            "DYLD_FRAMEWORK_PATH",
            "LD_LIBRARY_PATH",
        ):
            environment.pop(name)
    environment["PATH"] = os.pathsep.join(
        path for path in environment["PATH"].split(os.pathsep) if path not in runtime_dirs
    )
    result = subprocess.run(
        [str(executable), *arguments],
        input=source,
        text=True,
        capture_output=True,
        env=environment,
        cwd=executable.parent,
        timeout=30,
    )
    assert result.returncode == 0, (result.stdout, result.stderr)
    return result.stdout


def test_native_runtime_and_javascript(installed_cli):
    checks = json.loads(invoke(installed_cli, "doctor"))
    for name in ("javascriptCore", "lexbor", "libcurlImpersonate", "crypto"):
        assert checks[name] == "ok"
    assert json.loads(invoke(installed_cli, "run", "-", source="Promise.resolve(6 * 7)")) == 42


def test_bundled_extraction(installed_cli):
    class Article(BaseHTTPRequestHandler):
        def do_GET(self):
            body = b"<article><h1>Article</h1><p id='content'>Waiting</p></article><script>document.querySelector('#content').textContent='Standalone CLI fetched and rendered this article using only the released archive.'</script>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    with ThreadingHTTPServer(("127.0.0.1", 0), Article) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            output = invoke(
                installed_cli,
                "fetch",
                f"http://127.0.0.1:{server.server_port}/",
                "--format",
                "markdown",
            )
        finally:
            server.shutdown()
            thread.join()
    assert "Standalone CLI" in output
