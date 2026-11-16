"""Requests-style entry points, URL construction, and private-session ownership."""

import gc
import json
import threading
import weakref
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

import brimp
from brimp._urls import build_url


@pytest.fixture(scope="module")
def server():
    class Handler(BaseHTTPRequestHandler):
        def respond(self):
            body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
            if self.path == "/html":
                self.send_response(200)
                self.send_header("Content-Type", "text/html")
                self.end_headers()
                self.wfile.write(
                    b"<h1>original</h1><script>"
                    b"document.querySelector('h1').textContent = 'rendered'</script>"
                )
                return
            payload = json.dumps(
                dict(
                    method=self.command,
                    path=self.path,
                    body=body.decode(),
                    headers=dict(self.headers),
                )
            ).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            if self.command != "HEAD":
                self.wfile.write(payload)

        do_GET = do_HEAD = do_POST = do_PUT = do_PATCH = respond
        do_DELETE = do_OPTIONS = do_TRACE = do_QUERY = respond

        def log_message(self, *_):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{httpd.server_port}"
    httpd.shutdown()
    httpd.server_close()
    thread.join()


@pytest.mark.parametrize(
    "method", ["get", "head", "post", "put", "patch", "delete", "options", "trace", "query"]
)
def test_module_helpers_return_live_pages(server, method):
    kwargs = {"params": {"q": "a b"}, "headers": {"X-Test": "helper"}}
    if method in {"post", "put", "patch", "query"}:
        kwargs["body"] = "payload"
    with getattr(brimp, method)(server + "/echo", **kwargs) as page:
        assert isinstance(page, brimp.Page)
        assert page.status_code == 200
        assert page.request_info["method"] == method.upper()
        assert page.evaluate("location.pathname + location.search") == "/echo?q=a+b"
        session = page._session
        if method == "head":
            assert page.content == b""
        else:
            data = page.json()
            assert data["method"] == method.upper()
            assert data["path"] == "/echo?q=a+b"
            assert data["body"] == kwargs.get("body", "")
            assert {key.lower(): value for key, value in data["headers"].items()}[
                "x-test"
            ] == "helper"
        # A helper's private session stays alive for later navigation and JS.
        assert page.get(server + "/again") is page
        assert page.evaluate("document.URL") == server + "/again"
    assert page.json()["path"] == "/again"
    page.close()
    with pytest.raises(brimp.Closed):
        session.new_page()


def test_module_request_cleans_up_after_collection_and_failure(server, monkeypatch):
    sessions = []

    class TrackingSession(brimp.Session):
        def __init__(self):
            super().__init__(plugins=[])
            sessions.append(self)
            self.close_count = 0

        def close(self):
            self.close_count += 1
            super().close()

    monkeypatch.setattr(brimp, "Session", TrackingSession)
    page = brimp.request("GET", server)
    reference = weakref.ref(page)
    del page
    gc.collect()
    assert reference() is None
    assert sessions[-1].close_count == 1
    with pytest.raises(brimp.Closed):
        sessions[-1].new_page()

    for kwargs, error in [
        ({"unknown": True}, TypeError),
        ({"body": "a", "data": {"b": 1}}, brimp.InvalidInput),
    ]:
        with pytest.raises(error):
            brimp.get(server, **kwargs)
        assert sessions[-1].close_count == 1
        with pytest.raises(brimp.Closed):
            sessions[-1].new_page()
    with pytest.raises(brimp.MissingSchema):
        brimp.get("not an absolute URL")
    assert sessions[-1].close_count == 1

    with pytest.raises(ValueError, match="caller failed"):
        with brimp.get(server) as page:
            raise ValueError("caller failed")
    page.close()
    del page
    gc.collect()
    assert sessions[-1].close_count == 1


def test_module_helpers_isolate_cookies_and_keep_javascript_alive(server):
    with brimp.get(server) as page:
        page.evaluate("document.cookie = 'helper=private; path=/'")
        page.get(server + "/again")
        headers = {key.lower(): value for key, value in page.json()["headers"].items()}
        assert "helper=private" in headers["cookie"]
        page.get(server + "/html")
        assert "<h1>rendered</h1>" in page.html()
        assert "<h1>original</h1>" in page.text
        with brimp.get(server) as other:
            headers = {key.lower(): value for key, value in other.json()["headers"].items()}
            assert "helper=private" not in headers.get("cookie", "")


def test_session_defaults_page_navigation_and_streaming(server):
    with brimp.Session(
        base_url=server + "/api/", params={"key": "session", "blank": ""}
    ) as session:
        with session.get("items?key=url", params={"key": "request", "tag": ["a", "b"]}) as page:
            expected = "/api/items?key=request&blank=&tag=a&tag=b"
            assert page.json()["path"] == expected
            assert page.request_info["url"] == server + expected
            session.params["key"] = "updated"
            session.base_url = server + "/changed/"
            assert page.trace("next").json()["path"] == "/changed/next?key=updated&blank="
            assert page.query("/root").json()["method"] == "QUERY"
            with page.stream("GET", "./file:1", params={"key": "stream"}, quote=":") as stream:
                assert json.loads(b"".join(stream))["path"] == "/changed/file%3A1?key=stream&blank="
        # Closing a regular page must leave its shared session open.
        with session.options("more") as page:
            assert page.status_code == 200


@pytest.mark.parametrize(
    "url,kwargs,expected",
    [
        ("/p?k=old&empty=#frag", {"params": {"k": "new", "n": 2}}, "/p?k=new&empty=&n=2#frag"),
        ("/p?k=1&k=2", {"params": {"k": 3}}, "/p?k=1&k=2&k=3"),
        ("/p?k=old", {"params": [("k", "a"), ("k", "b")]}, "/p?k=old&k=a&k=b"),
        (
            "/p",
            {"params": {"k": [1, 2], "flag": False, "obj": {"a": 1}}},
            "/p?k=1&k=2&flag=false&obj=%7B%22a%22%3A+1%7D",
        ),
        (
            "/p?k=url",
            {"default_params": {"k": "session"}, "params": {"k": "request"}},
            "/p?k=request",
        ),
        ("/p", {"params": {"none": None, "empty": [], "zero": 0}}, "/p?none=None&zero=0"),
        ("", {"base_url": "http://example.test/a/b?old=1"}, "http://example.test/a/b?old=1"),
        (
            "",
            {"base_url": "http://example.test/a/b?old=1", "params": {"new": 2}},
            "http://example.test/a/b?new=2",
        ),
        ("../c", {"base_url": "http://example.test/a/b/"}, "http://example.test/a/c"),
        ("/c", {"base_url": "http://example.test/a/b"}, "http://example.test/c"),
        ("//other.test/c", {"base_url": "http://example.test/a/"}, "http://other.test/c"),
        ("http://other.test/c", {"base_url": "http://example.test/a/"}, "http://other.test/c"),
        (
            "/c",
            {"base_url": "http://example.test/a/?old=1", "params": {"new": 2}},
            "http://example.test/c?new=2",
        ),
        ("/a b/测试?q=é#space here", {}, "/a%20b/%E6%B5%8B%E8%AF%95?q=%C3%A9#space%20here"),
        ("/%2f/%7e?q=%3A%7C", {}, "/%2f/~?q=%3A%7C"),
        ("/page:7?tags=id:<1000&blank=", {"quote": ":"}, "/page%3A7?tags=id%3A%3C1000&blank="),
        ("/query={} ", {"quote": False}, "/query={} "),
        ("/%7e", {"quote": False}, "/%7e"),
        ("/a b", {"quote": None}, "/a%20b"),
        ("/%zz/%2F", {}, "/%25zz/%252F"),
        ("/%/x%2", {}, "/%/x%2"),
        ("/p", {"params": {"q": "a b"}, "quote": False}, "/p?q=a+b"),
    ],
)
def test_url_construction(url, kwargs, expected):
    assert build_url(url, **kwargs) == expected


@pytest.mark.parametrize(
    "path,quote,expected",
    [
        ("/a b/测试?q=é", "", "/a%20b/%E6%B5%8B%E8%AF%95?q=%C3%A9"),
        ("/%2F/%7e?q=%3A%7C", "", "/%2F/~?q=%3A%7C"),
        ("/page:7?tags=id:<1000", ":", "/page%3A7?tags=id%3A%3C1000"),
        ("/query={}", False, "/query={}"),
    ],
)
def test_quoting_on_the_wire(server, path, quote, expected):
    with brimp.get(server + path, quote=quote) as page:
        assert page.json()["path"] == expected
        assert page.request_info["url"] == server + expected


def test_base_url_validation():
    with pytest.raises(ValueError, match="absolute URL"):
        brimp.Session(base_url="/relative")
    with brimp.Session(plugins=[]) as session:
        session.base_url = "relative"
        with pytest.raises(ValueError, match="absolute URL"):
            session.get("path")
