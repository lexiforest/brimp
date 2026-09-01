import json
import gc
import os
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import brimp
import pytest


class Handler(BaseHTTPRequestHandler):
    observed = []

    def _handle(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length)
        type(self).observed.append((self.command, self.path, dict(self.headers.items()), body))
        route = self.path.split("?", 1)[0]
        if route == "/missing":
            status, content_type = 404, "text/html; charset=utf-8"
            content = b"<main id='value'>raw</main><script>previousRealm = true; document.getElementById('value').textContent = 'rendered'</script>"
        elif route == "/next":
            status, content_type = 200, "text/html"
            content = b"<main id='realm'></main><script>document.getElementById('realm').textContent = typeof previousRealm</script>"
        elif route == "/json":
            status, content_type, content = 200, "application/json; charset=utf-8", b'{"answer":42}'
        elif route == "/redirect":
            self.send_response(302)
            self.send_header("Location", "/final")
            self.send_header("Set-Cookie", "redirected=yes; Path=/")
            self.end_headers()
            return
        elif route == "/post-redirect":
            self.send_response(302)
            self.send_header("Location", "/echo")
            self.end_headers()
            return
        elif route == "/final":
            status, content_type, content = 200, "text/html", b"<main>final</main>"
        elif route == "/echo":
            status, content_type = 200, "application/json"
            content = json.dumps(
                {
                    "method": self.command,
                    "body": body.decode(),
                    "contentType": self.headers.get("Content-Type"),
                    "authorization": self.headers.get("Authorization"),
                    "referer": self.headers.get("Referer"),
                }
            ).encode()
        else:
            status, content_type, content = 200, "text/html", b"<title>Top level</title>"
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(content)))
        self.send_header("Set-Cookie", "server=ready; Path=/")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(content)

    do_GET = _handle
    do_HEAD = _handle
    do_POST = _handle
    do_PUT = _handle
    do_PATCH = _handle
    do_DELETE = _handle
    do_OPTIONS = _handle

    def log_message(self, *_args):
        return


@pytest.fixture(scope="module")
def server_url():
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{server.server_port}"
    server.shutdown()
    thread.join()


def test_page_is_the_navigation_result_and_live_document(server_url):
    with brimp.Session(headers={"X-Test": "session"}, cookies={"manual": "yes"}, params={"base": "yes", "q": "old"}) as session:
        page = session.get(
            server_url + "/missing",
            params={"q": ["one", "two"]},
            headers=[("X-Dupe", "one"), ("X-Dupe", "two")],
        )
        assert isinstance(page, brimp.Page)
        assert page.status_code == 404
        assert page.reason == "Not Found"
        assert page.headers["CONTENT-TYPE"] == "text/html; charset=utf-8"
        assert ">raw<" in page.text
        assert ">rendered<" in page.html
        page.evaluate("document.getElementById('value').textContent = 'changed'")
        assert ">changed<" in page.html
        assert not page.ok
        assert page.elapsed > 0
        assert page.http_version in {"HTTP/1.0", "HTTP/1.1", "HTTP/2", "HTTP/3"}
        assert page.downloaded_bytes == len(page.content)
        assert page.uploaded_bytes == 0
        assert page.header_bytes > 0
        with pytest.raises(brimp.HTTPError) as raised:
            page.raise_for_status()
        assert raised.value.page is page
        assert page.cookies["server"] == "ready"
        assert page.last_request.method == "GET"
        assert page.last_request.url == server_url + "/missing?base=yes&q=one&q=two"
        assert page.last_request.headers.get_all("x-dupe") == ("one", "two")

        assert page.get(server_url + "/next") is page
        assert ">undefined<" in page.html
        _, _, headers, _ = Handler.observed[-1]
        lowered = {name.lower(): value for name, value in headers.items()}
        assert lowered["x-test"] == "session"
        assert "manual=yes" in lowered["cookie"]
        assert "server=ready" in lowered["cookie"]

        assert page.get(server_url + "/json") is page
        assert page.json() == {"answer": 42}
        assert page.html is None
        with pytest.raises(brimp.InvalidRequest):
            page.get(server_url, headers={"User-Agent": "incoherent"})

    with pytest.raises(brimp.SessionClosed):
        page.evaluate("document.title")


def test_request_bodies_verbs_auth_and_iteration(server_url):
    with brimp.Session(auth=("agent", "secret"), referer="https://ref.test/") as session:
        page = session.post(server_url + "/echo", data={"name": "Luke", "tag": ["a", "b"]})
        payload = page.json()
        assert page.uploaded_bytes == len(page.last_request.body)
        assert payload == {
            "method": "POST",
            "body": "name=Luke&tag=a&tag=b",
            "contentType": "application/x-www-form-urlencoded",
            "authorization": "Basic YWdlbnQ6c2VjcmV0",
            "referer": "https://ref.test/",
        }
        for method in ("PUT", "PATCH", "DELETE", "OPTIONS"):
            page.request(method, server_url + "/echo", json={"method": method})
            assert page.json()["method"] == method
            assert page.last_request.body == f'{{"method":"{method}"}}'.encode()

        page.head(server_url + "/echo")
        assert page.last_request.method == "HEAD"
        assert page.content == b""

        page.get(server_url + "/json")
        assert b"".join(page.iter_content(4)) == page.content
        assert list(page.iter_lines()) == [page.content]
        page.encoding = "utf-8"
        assert page.encoding == "utf-8"
        with pytest.raises(brimp.InvalidRequest):
            page.get(server_url, content=b"body")
        with pytest.raises(brimp.InvalidRequest):
            page.post(server_url, data={}, json={})


def test_redirect_controls_history_and_method_rewrite(server_url):
    with brimp.Session() as session:
        page = session.get(server_url + "/redirect")
        assert page.url == server_url + "/final"
        assert page.redirect_count == 1
        assert page.history[0].status_code == 302
        assert page.history[0].request.method == "GET"
        assert session.cookies["redirected"] == "yes"

        page.get(server_url + "/redirect", allow_redirects=False)
        assert page.status_code == 302
        assert page.url == server_url + "/redirect"
        assert page.history == ()

        page.post(server_url + "/post-redirect", content=b"payload")
        assert page.json()["method"] == "GET"
        assert page.last_request.method == "GET"
        assert page.last_request.body is None


def test_multipart_and_top_level_page_ownership(server_url):
    multipart = brimp.Multipart().addpart(
        name="attachment", data=b"payload", filename="value.txt", content_type="text/plain"
    )
    with brimp.post(server_url + "/echo", multipart=multipart) as page:
        assert page.json()["contentType"].startswith("multipart/form-data; boundary=")
        assert b'filename="value.txt"' in page.last_request.body
    with pytest.raises(brimp.SessionClosed):
        page.evaluate("document.title")


def test_explicit_page_screenshot_extract_and_subsystems(server_url):
    with brimp.Session() as session, tempfile.TemporaryDirectory() as directory:
        page = session.new_page()
        assert page.get(server_url) is page
        path = Path(directory) / "page.png"
        content = page.screenshot(path)
        assert path.read_bytes() == content
        assert content.startswith(b"\x89PNG\r\n\x1a\n")
        page.get(server_url + "/missing")
        assert "rendered" in page.extract(content_selector="#value")["contentMarkdown"]

    with brimp.Session() as session:
        page = session.new_page()
        assert page.evaluate(
            "[typeof Worker, typeof WebSocket, typeof indexedDB, typeof navigator.storage, typeof HTMLCanvasElement.prototype.getContext, 'gpu' in navigator, typeof AudioContext]"
        ) == ["undefined", "undefined", "undefined", "undefined", "undefined", False, "undefined"]


def test_session_page_pool_defaults_validation_and_reuse(server_url):
    with brimp.Session() as default_session:
        assert default_session.pool_size == 2 * (os.cpu_count() or 1)
    for invalid in (0, -1, True, 1.5, "2"):
        with pytest.raises(brimp.InvalidRequest):
            brimp.Session(pool_size=invalid)

    with brimp.Session(pool_size=1) as session:
        with session.get(server_url + "/missing") as first:
            native_page = first._inner
            first.evaluate("globalThis.poolLeak = 42")

        assert repr(first) == "<Page [released]>"
        with pytest.raises(brimp.PageReleased):
            _ = first.content
        with pytest.raises(brimp.PageReleased):
            first.evaluate("poolLeak")

        with session.get(server_url + "/next") as second:
            assert second._inner is native_page
            assert second.evaluate("typeof poolLeak") == "undefined"


def test_page_pool_blocks_and_exceptional_context_returns_lease(server_url):
    with brimp.Session(pool_size=1) as session:
        first = session.get(server_url)
        native_page = first._inner
        acquired = threading.Event()
        outcome = {}

        def use_waiting_page():
            with session.get(server_url + "/next") as second:
                outcome["same"] = second._inner is native_page
                acquired.set()

        worker = threading.Thread(target=use_waiting_page)
        worker.start()
        assert not acquired.wait(0.1)
        first.reset()
        assert acquired.wait(2)
        worker.join(2)
        assert not worker.is_alive()
        assert outcome == {"same": True}

        with pytest.raises(RuntimeError, match="handler failed"):
            with session.get(server_url) as page:
                native_page = page._inner
                raise RuntimeError("handler failed")
        with session.get(server_url) as replacement:
            assert replacement._inner is native_page


def test_page_pool_close_destruction_and_proxy_affinity(server_url):
    with brimp.Session(pool_size=1) as session:
        page = session.get(server_url)
        discarded = page._inner
        page.close()
        with pytest.raises(brimp.SessionClosed):
            page.evaluate("document.title")
        with session.get(server_url) as replacement:
            assert replacement._inner is not discarded

        abandoned = session.get(server_url)
        del abandoned
        gc.collect()
        with session.get(server_url) as recovered:
            assert recovered.status_code == 200

    with brimp.Session(pool_size=1) as session:
        proxied = session.new_page(proxy="http://127.0.0.1:9")
        proxied_native = proxied._inner
        proxied.reset()
        again = session.new_page(proxy="http://127.0.0.1:9")
        assert again._inner is proxied_native
        again.reset()
        with session.new_page() as direct:
            assert direct._inner is not proxied_native


def test_session_close_wakes_page_pool_waiters(server_url):
    session = brimp.Session(pool_size=1)
    page = session.get(server_url)
    waiting = threading.Event()
    outcome = {}

    def wait_for_page():
        waiting.set()
        try:
            session.get(server_url)
        except brimp.SessionClosed:
            outcome["closed"] = True

    worker = threading.Thread(target=wait_for_page)
    worker.start()
    assert waiting.wait(1)
    session.close()
    worker.join(2)
    assert not worker.is_alive()
    assert outcome == {"closed": True}
    with pytest.raises(brimp.SessionClosed):
        page.evaluate("document.title")
