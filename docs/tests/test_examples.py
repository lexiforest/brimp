"""Run tutorial fences against a local fixture and the actual built bindings."""

import importlib.util
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import threading
from http.server import ThreadingHTTPServer

import pytest

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs/src/content/docs"
EXPECTED = {
    "install.mdx": ["function", "function"],
    "quick-start.mdx": ["Hello from JavaScript", "Hello from JavaScript"],
    "examples/python.md": [
        "Hello from JavaScript",
        "javascript",
        "name=Ada",
        "experiment=rendered",
        "Hello, Ada",
        "en-US",
    ],
    "examples/node.md": [
        "Brimp tutorial",
        "Edited by Node",
        "Brimp tutorial",
        "Request cancelled",
        "experiment=rendered",
    ],
    "examples/fetch.md": ["Edited article", "Hello from JavaScript", "route", "53248", ""],
    "architecture/javascript-runtime.md": [
        "Hello from JavaScript",
        "Hello from JavaScript",
        "The previous realm is gone",
        "42",
    ],
    "architecture/subsystems.md": ["32", "host-storage", "42"],
}


@pytest.mark.parametrize("runner,filename", [("node", "run.mjs"), (sys.executable, "run.py")])
def test_script_debugging_example(runner, filename):
    result = subprocess.run(
        [runner, str(ROOT / "examples/harness" / filename)],
        env=dict(os.environ, PYTHONPATH=str(ROOT / "bindings/python/python")),
        text=True,
        capture_output=True,
        timeout=30,
    )
    assert result.returncode == 0, result.stderr
    output = json.loads(result.stdout)
    assert output["results"] == [
        dict(text="Hello Ada", calls=1, clicks=1, now=1005),
        dict(text="Hello Grace", calls=2, clicks=1, now=1010),
    ]
    assert output["report"]["coverage"]["enabled"] is True
    assert output["report"]["dropped"] == 0


def examples():
    for filename, expected in EXPECTED.items():
        blocks = re.findall(
            r"```(python|javascript|js)\n(.*?)\n```", (DOCS / filename).read_text(), re.S
        )
        assert len(blocks) == len(expected), f"Update tutorial checks for {filename}"
        for index, ((language, code), output) in enumerate(zip(blocks, expected)):
            yield pytest.param(language, code, output, id=f"{filename}:{index + 1}")


@pytest.fixture(scope="module")
def server():
    spec = importlib.util.spec_from_file_location(
        "tutorial_fixture", ROOT / "docs/fixtures/server.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    httpd = ThreadingHTTPServer(("127.0.0.1", 0), module.Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{httpd.server_port}"
    httpd.shutdown()
    httpd.server_close()
    thread.join()


@pytest.mark.parametrize("language,code,expected", list(examples()))
def test_tutorial(language, code, expected, server, tmp_path):
    code = code.replace("http://127.0.0.1:8765", server)
    env = dict(os.environ, PYTHONPATH=str(ROOT / "bindings/python/python"))
    if language == "python":
        command = [sys.executable, "-c", code]
    else:
        code = code.replace(
            "from '@brimp/brimp'", f"from '{(ROOT / 'bindings/node/index.js').as_uri()}'"
        )
        command = ["node", "--input-type=module", "-e", code]
    result = subprocess.run(
        command, cwd=tmp_path, env=env, text=True, capture_output=True, timeout=30
    )
    assert result.returncode == 0, result.stderr
    assert expected in result.stdout, result.stdout
    if "download.bin" in code:
        assert (tmp_path / "download.bin").read_bytes() == b"Brimp stream\n" * 4096
    if "example.html" in code:
        assert "Brimp tutorial" in (tmp_path / "example.html").read_text()


def test_cli_tutorial(server, tmp_path):
    executable = Path(
        os.environ.get(
            "BRIMP_CLI", ROOT / "target/debug" / ("brimp.exe" if os.name == "nt" else "brimp")
        )
    ).resolve()
    assert executable.is_file(), "Build brimp-cli before testing tutorials"
    # Live browser capture is verified separately; provide its partial document here.
    (tmp_path / "browser-profile.json").write_text(
        '{"format":"brimp-profile","version":1,"values":{}}'
    )
    # Run the shell blocks exactly as printed; use a temporary output directory.
    for source in re.findall(r"```sh\n(.*?)\n```", (DOCS / "examples/cli.md").read_text(), re.S):
        source = source.replace("http://127.0.0.1:8765", server)
        source = re.sub(
            r"\bbrimp(?= (doctor|fetch|run))", shlex.quote(executable.as_posix()), source
        )
        result = subprocess.run(
            ["sh", "-eu", "-c", source],
            cwd=tmp_path,
            env=dict(os.environ, MSYS2_ARG_CONV_EXCL="*"),
            text=True,
            capture_output=True,
            timeout=30,
        )
        assert result.returncode == 0, result.stderr
        if "document.querySelector(" in source:
            assert "Hello from JavaScript" in result.stdout
        if "Local script" in source:
            assert "Local script" in result.stdout
    assert "Hello from JavaScript" in (tmp_path / "example.html").read_text()
    assert "Hello from JavaScript" in (tmp_path / "example.md").read_text()


def test_native_exception_stubs():
    """Keep the static reference's native exception stubs aligned with PyO3."""
    import ast

    tree = ast.parse((ROOT / "bindings/python/python/brimp/_brimp.pyi").read_text())
    stubs = {
        node.name: ast.unparse(node.bases[0])
        for node in tree.body
        if isinstance(node, ast.ClassDef)
    }
    native = dict(
        re.findall(
            r"create_exception!\(_brimp, (\w+), (\w+)\)",
            (ROOT / "bindings/python/src/lib.rs").read_text(),
        )
    )
    native = {name: "Exception" if base == "PyException" else base for name, base in native.items()}
    assert stubs == native


def test_cli_workflows(server, tmp_path):
    executable = Path(
        os.environ.get(
            "BRIMP_CLI", ROOT / "target/debug" / ("brimp.exe" if os.name == "nt" else "brimp")
        )
    ).resolve()
    guide = (DOCS / "examples/cli-workflows.md").read_text()
    for language, filename, code in re.findall(
        r'```(\w+)(?: title="([^"]+)")?\n(.*?)\n```', guide, re.S
    ):
        code = code.replace("http://127.0.0.1:8765", server)
        if filename:
            (tmp_path / filename).write_text(code)
        elif language == "sh":
            code = re.sub(
                r"\bbrimp(?= (fetch|run|plugins))", shlex.quote(executable.as_posix()), code
            )
            result = subprocess.run(
                ["sh", "-eu", "-c", code],
                cwd=tmp_path,
                env=dict(os.environ, MSYS2_ARG_CONV_EXCL="*"),
                text=True,
                capture_output=True,
                timeout=30,
            )
            assert result.returncode == 0, result.stderr
            if "--actions" in code:
                assert "Hello, Ada" in result.stdout
            if "--routes" in code:
                assert '"source":"route"' in result.stdout
            if "--plugin ./custom.js" in code:
                assert result.stdout.strip() == "42"
            if "--clock logical" in code:
                assert result.stdout.strip() == "[1700000000050]"
    assert (tmp_path / "streamed.bin").read_bytes() == b"Brimp stream\n" * 4096
    assert (tmp_path / "trace.jsonl").is_file()
    assert (tmp_path / "events.jsonl").is_file()
