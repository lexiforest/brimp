"""Process-boundary acceptance tests. Run after building brimp and lite-worker."""
import asyncio
import contextlib
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import time

import pytest

from cdp_proxy_test import run_test
from cdp_smoke_test import RawCDPClient, get_json, run as webkit_smoke

pytestmark = pytest.mark.skipif(sys.platform != "darwin", reason="controller host adapter is macOS-only")
ROOT = Path(__file__).resolve().parents[3]
CLI = Path(os.environ.get("BRIMP_TEST_CLI", ROOT / "target/debug/brimp"))
WORKER = ROOT / "target/debug/lite-worker"


def test_managed_cdp_proxy():
    asyncio.run(run_test(str(CLI)))


@contextlib.contextmanager
def controller(worker):
    process = subprocess.Popen([str(CLI), "serve", "--worker-path", str(worker), "--pool-size=3", "--port=0", "--max-worker-memory-mb=0"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        assert select.select([process.stdout], [], [], 15)[0], "controller startup timed out"
        line = process.stdout.readline()
        assert line.startswith("Brimp listening on "), line
        yield line.removeprefix("Brimp listening on ").strip()
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def test_lite_public_cdp_workflow():
    async def workflow(base):
        version = await get_json(base, "/json/version")
        client = await RawCDPClient.connect(version["webSocketDebuggerUrl"])
        try:
            context = (await client.command("Target.createBrowserContext"))["browserContextId"]
            target = (await client.command("Target.createTarget", {"url": "about:blank", "browserContextId": context}))["targetId"]
            session = (await client.command("Target.attachToTarget", {"targetId": target, "flatten": True}))["sessionId"]
            await client.command("Page.enable", session_id=session)
            await client.command("Runtime.enable", session_id=session)
            await client.command("Page.navigate", {"url": "data:text/html,<title>Worker</title><h1>Hello</h1>"}, session)
            await client.wait_for_event("Page.loadEventFired", session_id=session)
            value = await client.command("Runtime.evaluate", {"expression": "document.title", "returnByValue": True}, session)
            assert value["result"]["value"] == "Worker"
            screenshot = await client.command("Page.captureScreenshot", {"format": "png"}, session)
            assert screenshot["data"].startswith("iVBOR")
            await client.command("Target.disposeBrowserContext", {"browserContextId": context})
            assert context not in (await client.command("Target.getBrowserContexts"))["browserContextIds"]
        finally:
            await client.close()
    with controller(WORKER) as base:
        asyncio.run(workflow(base))


def managed_arguments(tmp_path, mode="normal"):
    record = tmp_path / "worker.json"
    log = tmp_path / "commands.jsonl"
    arguments = [str(Path(__file__).with_name("fake_cdp_worker.py")),
                 "--record=" + str(record), "--log=" + str(log), "--mode=" + mode]
    args_file = tmp_path / "worker args.json"
    args_file.write_text(json.dumps(arguments))
    args = ["--worker-path", sys.executable, "--cdp", "--worker-args", str(args_file)]
    return args, record, log


def assert_managed_worker_reaped(record):
    worker = json.loads(record.read_text())
    with pytest.raises(ProcessLookupError):
        os.kill(worker["pid"], 0)
    assert not Path(worker["profile"]).exists()


@pytest.mark.parametrize("mode", ["normal", "discovery-keepalive"])
def test_cli_launches_websocket_worker_and_reaps_it(tmp_path, mode):
    args, record, log = managed_arguments(tmp_path, mode)
    result = subprocess.run([str(CLI), "fetch", "https://unused.test/", "--eval", "document.title", *args], capture_output=True, text=True, timeout=10)
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == "Managed"
    methods = [json.loads(line)["method"] for line in log.read_text().splitlines()]
    assert "Page.navigate" in methods
    assert "Target.disposeBrowserContext" in methods
    assert_managed_worker_reaped(record)


@pytest.mark.parametrize("mode,exit_code", [("startup-crash", 10), ("startup-hang", 14), ("discovery-hang", 14), ("handshake-hang", 14), ("command-hang", 14), ("command-crash", 10), ("bad-version", 16)])
def test_managed_websocket_failure_reaps_worker(tmp_path, mode, exit_code):
    args, record, _ = managed_arguments(tmp_path, mode)
    result = subprocess.run([str(CLI), "fetch", "https://unused.test/", "--timeout", "2s", *args], capture_output=True, text=True, timeout=8)
    assert result.returncode == exit_code, result.stderr
    assert_managed_worker_reaped(record)


@pytest.mark.parametrize("mode", ["startup-hang", "handshake-hang", "command-hang"])
def test_managed_websocket_cancellation_reaps_worker(tmp_path, mode):
    args, record, _ = managed_arguments(tmp_path, mode)
    process = subprocess.Popen([str(CLI), "fetch", "https://unused.test/", *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        deadline = time.monotonic() + 5
        while not record.exists():
            assert process.poll() is None
            assert time.monotonic() < deadline
            time.sleep(0.01)
        time.sleep(0.2)
        process.send_signal(signal.SIGINT)
        _, stderr = process.communicate(timeout=5)
        assert process.returncode == 15, stderr
        assert_managed_worker_reaped(record)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


@pytest.mark.parametrize("option,value", [("--connect", "ws://127.0.0.1:9222"), ("--named-pipe", "unused"), ("--worker-protocol", "cdp")])
@pytest.mark.parametrize("command", ["doctor", "serve"])
def test_removed_worker_selection_options_are_rejected(command, option, value):
    result = subprocess.run([str(CLI), command, "--worker-path", "/unused", option, value], capture_output=True, timeout=5)
    assert result.returncode == 2


def test_webkit_acceptance():
    worker = os.environ.get("BRIMP_TEST_WEBKIT_WORKER")
    if not worker:
        pytest.skip("set BRIMP_TEST_WEBKIT_WORKER to the matching WebKit worker")
    with controller(worker) as base:
        asyncio.run(webkit_smoke(base, False))


FAKE = r'''
import json, os, socket, struct, sys, time
fd = int(next(arg.split("=", 1)[1] for arg in sys.argv if arg.startswith("--controller-socket-fd=")))
sock = socket.socket(fileno=fd)
mode = os.environ["BRIMP_FAKE_MODE"]
open(os.environ["BRIMP_FAKE_PID"], "w").write(str(os.getpid()))
header = sock.recv(4)
length = struct.unpack("!I", header)[0]
data = b""
while len(data) < length:
    data += sock.recv(length - len(data))
request = json.loads(data)
if mode == "hang":
    time.sleep(60)
elif mode == "crash":
    sys.exit(7)
elif mode == "oversized":
    sock.sendall(struct.pack("!I", 64 * 1024 * 1024 + 1))
elif mode == "truncated":
    sock.sendall(struct.pack("!I", 10) + b"{")
elif mode == "invalid":
    sock.sendall(struct.pack("!I", 1) + b"!")
elif mode == "array":
    sock.sendall(struct.pack("!I", 2) + b"[]")
elif mode == "unsupported":
    reply = json.dumps({"id": request["id"], "result": {"protocolVersion":"1.3", "product":"WebKitAutomationWorker/0.1"}}).encode()
    sock.sendall(struct.pack("!I", len(reply)) + reply)
    time.sleep(60)
'''


@pytest.fixture
def fake(tmp_path):
    worker = tmp_path / "fake-worker"
    worker.write_text(f"#!{sys.executable}\n" + FAKE)
    worker.chmod(0o755)
    pid = tmp_path / "pid"
    return worker, pid


def assert_reaped(pid_file):
    pid = int(pid_file.read_text())
    with pytest.raises(ProcessLookupError):
        os.kill(pid, 0)


@pytest.mark.parametrize("mode", ["crash", "oversized", "truncated", "invalid", "array"])
def test_bad_worker_frames_fail_and_reap_child(fake, mode):
    worker, pid = fake
    env = {**os.environ, "BRIMP_FAKE_MODE": mode, "BRIMP_FAKE_PID": str(pid)}
    result = subprocess.run([str(CLI), "doctor", "--worker-path", str(worker)], env=env, capture_output=True, timeout=8)
    assert result.returncode == 10, result.stderr
    assert result.stdout == b""
    assert_reaped(pid)


def test_startup_deadline_reaps_child(fake):
    worker, pid = fake
    env = {**os.environ, "BRIMP_FAKE_MODE":"hang", "BRIMP_FAKE_PID":str(pid)}
    result = subprocess.run([str(CLI), "fetch", "https://unused.test/", "--eval", "1", "--timeout", "2s", "--worker-path", str(worker)], env=env, capture_output=True, timeout=8)
    assert result.returncode == 14, result.stderr
    assert_reaped(pid)


def test_unsupported_worker_options_fail_before_navigation(fake):
    worker, pid = fake
    env = {**os.environ, "BRIMP_FAKE_MODE":"unsupported", "BRIMP_FAKE_PID":str(pid)}
    result = subprocess.run([str(CLI), "fetch", "https://unused.test/", "--eval", "1", "--proxy", "http://127.0.0.1:8080", "--worker-path", str(worker)], env=env, capture_output=True, timeout=8)
    assert result.returncode == 16, result.stderr
    assert_reaped(pid)


@pytest.mark.parametrize("sig", [signal.SIGINT, signal.SIGTERM])
@pytest.mark.parametrize("command", ["doctor", "serve"])
def test_signal_during_startup_reaps_child(fake, sig, command):
    worker, pid = fake
    env = {**os.environ, "BRIMP_FAKE_MODE":"hang", "BRIMP_FAKE_PID":str(pid)}
    process = subprocess.Popen([str(CLI), command, "--worker-path", str(worker)], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        deadline = time.monotonic() + 5
        while not pid.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        assert pid.exists()
        process.send_signal(sig)
        _, stderr = process.communicate(timeout=8)
        assert process.returncode == 15, stderr
        assert_reaped(pid)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


def test_webkit_cli_workflows(tmp_path):
    import http.server
    import threading
    worker = os.environ.get("BRIMP_TEST_WEBKIT_WORKER")
    if not worker:
        pytest.skip("set BRIMP_TEST_WEBKIT_WORKER to the matching WebKit worker")

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            body = b"<html><head><title>WebKit CLI</title></head><body><article><h1>WebKit</h1><p>Worker extraction works.</p></article></body></html>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *args):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        url = f"http://127.0.0.1:{server.server_port}/"
        def run(*args):
            result = subprocess.run([str(CLI), *args, "--worker-path", worker], capture_output=True, timeout=40)
            assert result.returncode == 0, result.stderr
            return result.stdout
        assert json.loads(run("fetch", url, "--eval", "document.title")) == "WebKit CLI"
        assert b"Worker extraction works." in run("fetch", url, "--format", "markdown")
        screenshot = tmp_path / "page.png"
        run("fetch", url, "--output", str(screenshot))
        assert screenshot.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
        destination = tmp_path / "crawl"
        run("crawl", url, "--ignore-robots", "--depth", "0", "--output-dir", str(destination))
        assert b"Worker extraction works." in (destination / "index.md").read_bytes()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def test_lite_configuration_applies_custom_headers():
    import http.server
    import threading
    seen = []

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            seen.extend([self.headers.get("X-CLI-One"), self.headers.get("X-CLI-Two")])
            body = b"<title>Headers</title>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *args):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        result = subprocess.run([str(CLI), "fetch", f"http://127.0.0.1:{server.server_port}/", "--worker-path", str(WORKER), "--header", "X-CLI-One: first", "--header", "X-CLI-Two: second", "--eval", "document.title"], capture_output=True, timeout=10)
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout) == "Headers"
        assert seen == ["first", "second"]
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


@pytest.mark.parametrize("command", ["doctor", "fetch", "crawl", "serve"])
@pytest.mark.parametrize("contents", [None, "not json", '{}', '["valid", 42]'])
def test_invalid_worker_args_file_is_rejected_before_launch(tmp_path, command, contents):
    args_file = tmp_path / "arguments.json"
    if contents is not None:
        args_file.write_text(contents)
    args = [str(CLI), command, "--worker-path", "/must-not-launch", "--worker-args", str(args_file)]
    if command in ("fetch", "crawl"):
        args.append("https://unused.test/")
    result = subprocess.run(args, capture_output=True, text=True, timeout=5)
    assert result.returncode == 2, result.stderr
    assert "worker arguments" in result.stderr


def test_worker_args_file_preserves_strings_and_precedes_inline_arguments(tmp_path):
    args, record, _ = managed_arguments(tmp_path)
    args_file = tmp_path / "worker args.json"
    file_arguments = json.loads(args_file.read_text()) + ['--label=file value with "quotes"']
    args_file.write_text(json.dumps(file_arguments))
    inline = '--label=inline value with "quotes"'
    result = subprocess.run([str(CLI), "doctor", *args, "--worker-arg", inline], capture_output=True, text=True, timeout=10)
    assert result.returncode == 0, result.stderr
    actual = json.loads(record.read_text())["arguments"]
    # Python consumes the script path; the worker sees every following argument intact.
    expected = file_arguments[1:] + [inline]
    assert actual[:len(expected)] == expected
    assert_managed_worker_reaped(record)


def test_default_managed_worker_accepts_argument_file(tmp_path):
    args_file = tmp_path / "arguments.json"
    args_file.write_text('["--headless"]')
    result = subprocess.run([str(CLI), "doctor", "--worker-path", str(WORKER), "--worker-args", str(args_file)], capture_output=True, text=True, timeout=10)
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout)
