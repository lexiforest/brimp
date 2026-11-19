"""Timeout formats and cancellation through the real native runtime."""

import asyncio
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

import brimp


@pytest.fixture(scope="module")
def server():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path.startswith("/delay/"):
                time.sleep(float(self.path.rsplit("/", 1)[1]))
            body = b"ok"
            if self.path == "/jobs":
                body = b"""<script>Promise.resolve().then(() => {
                    const stop = Date.now() + 150;
                    while (Date.now() < stop) {}
                    globalThis.done = true;
                });</script>"""
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            if self.path == "/body":
                time.sleep(0.2)
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def log_message(self, *_):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    worker = threading.Thread(target=httpd.serve_forever, daemon=True)
    worker.start()
    yield f"http://127.0.0.1:{httpd.server_port}"
    httpd.shutdown()
    httpd.server_close()
    worker.join()


@pytest.mark.parametrize("timeout", [1, 0.8, None, 0])
def test_session_formats_and_native_operations(server, timeout):
    with brimp.Session(timeout=timeout, trust_env=False) as session:
        page = session.get(server + "/delay/0.1")
        assert page.text == "ok"
        assert page.evaluate("new Promise(resolve => setTimeout(() => resolve(42), 30))") == 42
        with page.evaluate_handle("({value: 7})") as handle:
            assert handle.value() == {"value": 7}


@pytest.mark.parametrize("stream", [False, True])
# A fractional budget that still covers realm installation (~0.3 s) plus the
# 0.2 s route delay; 0.5 s was below the install cost itself.
@pytest.mark.parametrize("timeout", [None, 0, 2, 1.5])
def test_request_override_and_omitted_inheritance(server, timeout, stream):
    with brimp.Session(timeout=0.05, plugins=[], trust_env=False) as session:
        page = session.new_page()
        if stream:
            with pytest.raises(brimp.RequestTimeout):
                page.stream("GET", server + "/delay/0.2")
            with page.stream("GET", server + "/body", timeout=timeout) as response:
                assert b"".join(response) == b"ok"
        else:
            with pytest.raises(brimp.RequestTimeout):
                page.get(server + "/delay/0.2")
            assert page.get(server + "/delay/0.2", timeout=timeout).text == "ok"


@pytest.mark.parametrize("stream", [False, True])
def test_finite_override_on_unlimited_session(server, stream):
    with brimp.Session(timeout=None, plugins=[], trust_env=False) as session:
        page = session.new_page()
        with pytest.raises(brimp.RequestTimeout):
            if stream:
                with page.stream("GET", server + "/body", timeout=0.05) as response:
                    b"".join(response)
            else:
                page.get(server + "/delay/0.2", timeout=0.05)


def test_disabled_navigation_deadline_includes_promise_jobs(server):
    with brimp.Session(timeout=0.05, trust_env=False) as session:
        page = session.get(server + "/jobs", timeout=None)
        assert page.evaluate("done") is True


@pytest.mark.parametrize(
    "timeout",
    [
        -1,
        float("nan"),
        float("inf"),
        True,
        False,
        10**30,
        10**400,
        (1, 2),
        "1",
        (),
        (1,),
        (1, 2, 3),
        (1, None),
        (-1, 2),
        (1, float("inf")),
    ],
)
def test_invalid_formats(server, timeout):
    with pytest.raises(brimp.InvalidInput):
        brimp.Session(timeout=timeout, plugins=[])
    with brimp.Session(plugins=[]) as session:
        with pytest.raises(brimp.InvalidInput):
            session.get(server, timeout=timeout)


def test_async_inheritance_override_and_cancellation(server):
    async def run():
        async with brimp.AsyncSession(timeout=0.05, plugins=[], trust_env=False) as session:
            with pytest.raises(brimp.RequestTimeout):
                await session.get(server + "/delay/0.2")
            page = await session.get(server + "/delay/0.2", timeout=None)
            assert page.text == "ok"
            task = asyncio.create_task(page.get(server + "/delay/1", timeout=0))
            await asyncio.sleep(0.05)
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await asyncio.wait_for(task, 2)
            assert (await page.get(server, timeout=1)).text == "ok"

    asyncio.run(run())


def test_longer_timeout_is_not_cut_short_by_transport_default(server):
    with brimp.get(server + "/delay/30.2", timeout=35, trust_env=False) as page:
        assert page.text == "ok"
