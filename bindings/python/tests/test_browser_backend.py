"""Shared engine contracts; set BRIMP_WEBKIT_EXECUTABLE to test the local helper."""

import asyncio
import base64
import os
import shlex
import signal
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import brimp
import pytest


@pytest.fixture(params=["runtime", "webkit"])
def engine_options(request):
    options = {"engine": request.param, "plugins": [], "timeout": 15}
    if request.param == "webkit":
        executable = os.environ.get("BRIMP_WEBKIT_EXECUTABLE")
        if not executable:
            pytest.skip("BRIMP_WEBKIT_EXECUTABLE is not configured")
        options["executable_path"] = executable
    return options


@pytest.fixture
def webkit_options():
    executable = os.environ.get("BRIMP_WEBKIT_EXECUTABLE")
    if not executable:
        pytest.skip("BRIMP_WEBKIT_EXECUTABLE is not configured")
    return {"engine": "webkit", "executable_path": executable, "timeout": 15}


@pytest.fixture(scope="module")
def backend_server():
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
            if self.path == "/post-redirect":
                self.send_response(303)
                self.send_header("Location", "/page")
                self.end_headers()
                return
            result = b"<title>posted</title>" + base64.b64encode(body)
            self.send_response(201)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(result)))
            self.send_header("X-Client", self.headers.get("X-Client", ""))
            self.end_headers()
            self.wfile.write(result)

        def do_GET(self):
            if self.path == "/redirect":
                self.send_response(302)
                self.send_header("Location", "/page")
                self.end_headers()
                return
            body = (
                b"<meta charset=windows-1252><title>caf\xe9</title>"
                b"<input id=field style='width:150px;height:30px'><button id=go style='width:100px;height:40px'>Go</button>"
                b"<script>window.fromPage = 7; document.querySelector('#go').onclick = e => "
                b"{ window.clicked = e.isTrusted; };</script>"
            )
            if self.path == "/framed":
                body += b"<iframe src='/page?iframe'></iframe>"
            self.send_response(201)
            self.send_header("Content-Type", "text/html; charset=windows-1252")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Set-Cookie", "server_cookie=present; Path=/; HttpOnly")
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{server.server_port}"
    server.shutdown()
    server.server_close()
    thread.join()


def test_navigation_bytes_scripts_input_and_cookies(engine_options, backend_server):
    with brimp.Session(**engine_options) as session:
        page = session.new_page()
        page.get(backend_server + "/page")
        assert page.status_code == 201
        assert b"caf\xe9" in page.content
        assert page.evaluate("document.title") == "caf\xe9"
        assert page.evaluate("fromPage") == 7
        assert "<button" in page.html()
        page.locator("#field").fill("hello")
        assert page.evaluate("document.querySelector('#field').value") == "hello"
        page.locator("#go").click()
        assert page.evaluate("clicked") is True
        assert any(cookie["name"] == "server_cookie" for cookie in session.cookies)
        session.set_cookie(backend_server, "shared", "yes")
        other = session.new_page()
        other.get(backend_server + "/page")
        assert "shared=yes" in other.evaluate("document.cookie")
        session.clear_cookies()
        assert session.cookies == []


def test_binary_request_body_and_redirect_metadata(engine_options, backend_server):
    with brimp.Session(**engine_options) as session:
        page = session.new_page()
        body = bytes([0, 255, 128, 65])
        page.post(backend_server + "/posted", content=body, headers={"X-Client": "brimp"})
        assert page.content == b"<title>posted</title>" + base64.b64encode(body)
        assert ("x-client", "brimp") in [(name.lower(), value) for name, value in page.headers]
        assert page.request_info["body"] == body
        page.post(backend_server + "/post-redirect", content=body)
        assert page.request_info["method"] == "GET"
        assert page.request_info["body"] is None
        assert len(page.history) == 1


def test_handles_navigation_waits_and_private_state(engine_options, backend_server):
    with brimp.Session(**engine_options) as session:
        page = session.new_page()
        page.get(backend_server + "/page")
        handle = page.evaluate_handle("({value: 41})")
        assert handle.call("object => object.value + 1").value() == 42
        assert handle.json_value() == {"value": 41}
        handle.dispose()
        with pytest.raises(brimp.StaleHandle):
            handle.json_value()
        old = page.evaluate_handle("({old: true})")
        page.get(backend_server + "/page?again")
        with pytest.raises(brimp.StaleHandle):
            old.json_value()
        assert (
            page.evaluate("Object.getOwnPropertyNames(window).filter(k => k.startsWith('__brimp'))")
            == []
        )
        with pytest.raises(brimp.Timeout):
            page.wait_for_selector("#absent", timeout=0.05)
        assert page.evaluate("2 + 2") == 4
        page.close()
        assert page not in session.pages


def test_async_cancellation_keeps_page_usable(engine_options, backend_server):
    async def run():
        async with brimp.AsyncSession(**engine_options) as session:
            page = await session.new_page()
            await page.get(backend_server + "/page")
            task = asyncio.create_task(page.wait_for_function("false", timeout=None))
            await asyncio.sleep(0.05)
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await task
            assert await page.evaluate("42") == 42

    asyncio.run(run())


def test_document_identity_redirects_and_popup_ownership(engine_options, backend_server):
    with brimp.Session(**engine_options) as session:
        page = session.new_page()
        page.get(backend_server + "/framed")
        assert page.url == backend_server + "/framed"
        assert b"<iframe" in page.content
        page.get(backend_server + "/redirect")
        assert page.url == backend_server + "/page"
        assert len(page.history) == 1
        with page.expect_popup(timeout=10) as opened:
            page.evaluate(
                "void (window.child = window.open(" + repr(backend_server + "/page") + "))"
            )
        child = opened.value
        assert child in session.pages
        assert child.evaluate("document.title") == "caf\xe9"
        child.close()
        assert page.evaluate("child.closed") is True
        destination = backend_server.replace("127.0.0.1", "localhost") + "/page"
        page.get(destination)
        assert page.url == destination
        assert page.evaluate("document.title") == "caf\xe9"


def test_html_loading_skips_scripts_and_handles_remain_private(engine_options):
    with brimp.Session(**engine_options) as session:
        page = session.new_page()
        page.load_html(
            "<title>loaded</title><script>window.shouldNotRun = true</script>",
            url="https://fixture.test/document",
        )
        assert page.current_url() == "https://fixture.test/document"
        assert page.evaluate("typeof window.shouldNotRun") == "undefined"
        assert page.evaluate("document.title") == "loaded"
        page.evaluate("setTimeout(() => window.afterLoad = 42, 10)")
        assert page.wait_for_function("() => window.afterLoad", timeout=5) == 42
        assert page.evaluate("({value: 42, bytes: new Uint8Array([0, 255])})") == {
            "value": 42,
            "bytes": bytes([0, 255]),
        }


def test_webkit_stops_synchronous_infinite_script(webkit_options, backend_server):
    with brimp.Session(**webkit_options) as session:
        page = session.new_page()
        page.get(backend_server + "/page")
        started = time.monotonic()
        with pytest.raises(brimp.Timeout):
            page.evaluate("while (true) {}", timeout=0.1)
        assert time.monotonic() - started < 5
        with pytest.raises(brimp.Closed):
            page.evaluate("42")


def test_webkit_init_scripts_frames_and_session_isolation(webkit_options, backend_server):
    with brimp.Session(**webkit_options) as session:
        page = session.new_page()
        page.add_init_script("window.initializedBeforePage = typeof window.fromPage")
        page.get(backend_server + "/framed")
        assert page.evaluate("initializedBeforePage") == "undefined"
        assert page.evaluate("frames[0].initializedBeforePage") == "undefined"
        page.get(backend_server + "/page?again")
        assert page.evaluate("initializedBeforePage") == "undefined"
        other = session.new_page()
        other.get(backend_server + "/page")
        assert other.evaluate("'initializedBeforePage' in window") is False
        session.set_cookie(backend_server, "isolated", "yes")
        page.clear_cache()
        assert any(cookie["name"] == "isolated" for cookie in session.cookies)
        with brimp.Session(**webkit_options) as separate:
            assert separate.cookies == []
        session.clear_cookies()
        assert page.evaluate("document.cookie") == ""


def test_webkit_process_exit_rejects_pending_calls(webkit_options, backend_server, tmp_path):
    pid_file = tmp_path / "pid"
    wrapper = tmp_path / "webkit"
    wrapper.write_text(
        "#!/bin/sh\n"
        f"echo $$ > {shlex.quote(str(pid_file))}\n"
        f"exec {shlex.quote(webkit_options['executable_path'])}\n"
    )
    wrapper.chmod(0o755)
    webkit_options["executable_path"] = str(wrapper)
    with brimp.Session(**webkit_options) as session, ThreadPoolExecutor() as executor:
        page = session.new_page()
        page.get(backend_server + "/page")
        pending = executor.submit(page.wait_for_function, "false", timeout=None)
        time.sleep(0.05)
        os.kill(int(pid_file.read_text()), signal.SIGKILL)
        with pytest.raises(brimp.Closed):
            pending.result(timeout=5)
