"""Proxy policy tested against local origins and independently identified proxies."""

import asyncio
import base64
import json
import os
import threading
from contextlib import ExitStack, contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

import pytest

import brimp


@contextmanager
def endpoint(name):
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            path = urlsplit(self.path).path
            headers = {key.lower(): value for key, value in self.headers.items()}
            requests.append((self.path, headers))
            status, mime, extra = 200, "application/json", {}
            body = json.dumps(dict(server=name, path=path, headers=headers)).encode()
            if path == "/redirect":
                status, extra, body = 302, {"Location": "/final"}, b""
            elif path == "/html":
                mime = "text/html"
                body = (
                    b'<link rel="stylesheet" href="/style.css"><script src="/script.js"></script>'
                )
            elif path == "/script.js":
                mime, body = "text/javascript", b"globalThis.scriptLoaded = true;"
            elif path == "/style.css":
                mime, body = "text/css", b"body { color: red; }"
            elif path == "/events":
                mime, body = "text/event-stream", b"data: hello\n\n"
            elif path == "/cached":
                extra["Cache-Control"] = "public, max-age=3600"
            elif path == "/cookie":
                extra["Set-Cookie"] = "shared=yes; Path=/"
            self.send_response(status)
            self.send_header("Content-Type", mime)
            self.send_header("Content-Length", str(len(body)))
            for key, value in extra.items():
                self.send_header(key, value)
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}", requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


@pytest.fixture(scope="module")
def endpoints():
    with ExitStack() as stack:
        yield [stack.enter_context(endpoint(name)) for name in ("origin", "a", "b")]


@pytest.fixture(autouse=True)
def clean_proxy_environment(monkeypatch):
    for key in ("http_proxy", "https_proxy", "all_proxy", "no_proxy"):
        monkeypatch.delenv(key, raising=False)
        monkeypatch.delenv(key.upper(), raising=False)


def test_session_request_route_precedence_and_removal(endpoints):
    (origin, _), (a, _), (b, _) = endpoints
    with brimp.Session(proxy=a, plugins=[]) as session:
        page = session.get(origin)
        assert page.json()["server"] == "a"
        assert page.get(origin, proxy=b).json()["server"] == "b"
        assert page.get(origin, proxy="").json()["server"] == "origin"
        assert page.get(origin).json()["server"] == "a"
        remove = session.route("*", lambda route: route.continue_request(proxy=b))
        assert page.get(origin, proxy="").json()["server"] == "b"
        with session.get(origin) as other:
            assert other.json()["server"] == "b"
        off = page.route("*", lambda route: route.continue_request(proxy=""))
        assert page.get(origin).json()["server"] == "origin"
        off()
        assert page.get(origin).json()["server"] == "b"
        remove()
        assert page.get(origin).json()["server"] == "a"


def test_proxy_mapping_and_validation(endpoints):
    (origin, _), (a, _), (b, _) = endpoints
    with brimp.Session(proxies={"all": a, "http://127.0.0.1": b}, plugins=[]) as session:
        assert session.get(origin).json()["server"] == "b"
        assert session.get(origin, proxies={"http": a}).json()["server"] == "a"
        assert session.get(origin, proxies={"all://127.0.0.1": ""}).json()["server"] == "origin"
    with pytest.raises(brimp.InvalidInput, match="proxy or proxies"):
        brimp.Session(proxy=a, proxies={"http": b})
    secret = "do-not-expose-this-password"
    with pytest.raises(Exception) as error:
        brimp.Session(proxy=f"http://user:{secret}@[")
    assert secret not in str(error.value)


def test_proxy_auth_is_separate_and_does_not_follow_proxy_changes(endpoints):
    (origin, _), (a, _), (b, _) = endpoints
    token = "Basic " + base64.b64encode(b"proxy-user:proxy-pass").decode()
    with brimp.Session(proxy=a, proxy_auth=("proxy-user", "proxy-pass"), plugins=[]) as session:
        page = session.get(origin)
        assert page.json()["headers"]["proxy-authorization"] == token
        assert "authorization" not in page.json()["headers"]
        assert "proxy-authorization" not in page.get(origin, proxy="").json()["headers"]
        assert "proxy-authorization" not in page.get(origin, proxy=b).json()["headers"]
        session.route("*", lambda route: route.continue_request(proxy=b))
        assert "proxy-authorization" not in page.get(origin).json()["headers"]
        session.route(
            "*",
            lambda route: route.continue_request(proxy=b, proxy_auth=("proxy-user", "proxy-pass")),
        )
        assert page.get(origin).json()["headers"]["proxy-authorization"] == token


def test_environment_and_explicit_direct(endpoints, monkeypatch):
    (origin, _), (a, _), (b, _) = endpoints
    monkeypatch.setenv("http_proxy", a)
    with brimp.Session(plugins=[]) as session:
        assert session.get(origin).json()["server"] == "a"
        assert session.get(origin, trust_env=False).json()["server"] == "origin"
        assert session.get(origin, proxy="").json()["server"] == "origin"
        monkeypatch.setenv("no_proxy", "127.0.0.1")
        assert session.get(origin).json()["server"] == "origin"
        assert session.get(origin, proxy=b).json()["server"] == "b"
    with brimp.Session(trust_env=False, plugins=[]) as session:
        assert session.get(origin).json()["server"] == "origin"
    monkeypatch.delenv("http_proxy")
    monkeypatch.delenv("no_proxy")
    monkeypatch.setenv("HTTP_PROXY", a)
    # Windows treats HTTP_PROXY and http_proxy as the same environment key.
    expected = "a" if os.name == "nt" else "origin"
    assert brimp.get(origin).json()["server"] == expected
    assert brimp.get(origin, trust_env=False).json()["server"] == "origin"


def test_redirects_reapply_routes_and_request_proxy(endpoints):
    (origin, _), (a, _), (b, _) = endpoints
    with brimp.Session(plugins=[]) as session:
        page = session.get(origin + "/redirect", proxy=a)
        assert page.json()["server"] == "a"
        seen = []

        def policy(route):
            seen.append((route.request["url"], route.request["resource_type"]))
            route.continue_request(proxy=b if route.request["url"].endswith("/final") else a)

        session.route("*", policy)
        assert page.get(origin + "/redirect").json()["server"] == "b"
        assert seen == [(origin + "/redirect", "document"), (origin + "/final", "document")]


def test_stream_and_shared_cookies_across_proxies(endpoints):
    (origin, _), (a, _), (b, _) = endpoints
    with brimp.Session(proxy=a, plugins=[]) as session:
        page = session.get(origin + "/cookie")
        with page.stream("GET", origin, proxy=b) as response:
            result = json.loads(b"".join(response))
            assert result["server"] == "b"
            assert "shared=yes" in result["headers"]["cookie"]
        assert page.get(origin, proxy="").json()["server"] == "origin"
        assert "shared=yes" in page.json()["headers"]["cookie"]


def test_cache_is_partitioned_and_routes_cannot_be_bypassed(endpoints):
    (origin, _), (a, arequests), (b, brequests) = endpoints
    with brimp.Session(plugins=[]) as session:
        page = session.new_page()
        url = origin + "/cached"
        assert page.get(url, proxy=a).json()["server"] == "a"
        # The first navigation changes Sec-Fetch-Site from none to same-origin.
        assert page.get(url, proxy=a).json()["server"] == "a"
        count = len(arequests)
        assert page.get(url, proxy=a).json()["server"] == "a"
        assert len(arequests) == count
        assert page.get(url, proxy=b).json()["server"] == "b"
        count = len(brequests)
        assert page.get(url, proxy=b).json()["server"] == "b"
        assert len(brequests) == count
        off = session.route("*", lambda route: route.fail("filtered"))
        with pytest.raises(brimp.TransportError, match="filtered"):
            page.get(url, proxy=a)
        off()
        off = session.route(
            "*",
            lambda route: route.fulfill(body="injected", headers={"Cache-Control": "max-age=3600"}),
        )
        assert page.get(url, proxy=a).text == "injected"
        off()
        assert page.get(url, proxy=a).json()["server"] == "a"


def test_resource_types_and_main_document_override(endpoints):
    (origin, _), (a, arequests), (b, brequests) = endpoints
    with brimp.Session(proxy=a, plugins=[]) as session:
        seen = []

        def policy(route):
            seen.append((route.request["url"], route.request["resource_type"]))
            route.continue_request()

        session.route("*", policy)
        page = session.get(origin + "/html", proxy=b)
        assert page.evaluate("scriptLoaded") is True
        assert any(url.endswith("/script.js") for url, _ in arequests)
        assert not any(url.endswith("/script.js") for url, _ in brequests)
        assert page.evaluate("fetch('/data').then(r => r.json()).then(r => r.server)") == "a"
        assert (
            page.evaluate(
                "new Promise((resolve, reject) => {const x = new XMLHttpRequest(); x.open('GET', '/xhr'); x.onload = () => resolve(x.status); x.onerror = reject; x.send();})"
            )
            == 200
        )
        assert (
            page.evaluate(
                "new Promise((resolve, reject) => {const s = new EventSource('/events'); s.onmessage = e => { s.close(); resolve(e.data); }; s.onerror = reject;})"
            )
            == "hello"
        )
        types = {urlsplit(url).path: kind for url, kind in seen}
        assert types == {
            "/html": "document",
            "/script.js": "script",
            "/style.css": "stylesheet",
            "/data": "fetch",
            "/xhr": "xhr",
            "/events": "eventsource",
        }
        assert page.evaluate("typeof globalThis.__brimpSetResourceFetch") == "undefined"


def test_proxy_failure_does_not_fall_back_to_direct(endpoints):
    (origin, requests), _, _ = endpoints
    count = len(requests)
    with brimp.Session(plugins=[], timeout=1) as session:
        with pytest.raises(brimp.TransportError):
            session.get(origin, proxy="http://127.0.0.1:1")
    assert len(requests) == count


def test_async_concurrent_proxy_selection(endpoints):
    (origin, _), (a, _), (b, _) = endpoints

    async def run():
        async with brimp.AsyncSession(proxy=a, plugins=[], max_clients=4) as session:
            pages = await asyncio.gather(
                *(session.get(origin, proxy=p) for p in [a, b, "", b, a, ""])
            )
            assert [page.json()["server"] for page in pages] == [
                "a",
                "b",
                "origin",
                "b",
                "a",
                "origin",
            ]
            await session.route("*", lambda route: route.continue_request(proxy=b))
            page = await session.get(origin, proxy="")
            assert page.json()["server"] == "b"

    asyncio.run(run())


@contextmanager
def socket_server(handler):
    import socket

    errors = []
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen()
    listener.settimeout(5)

    def serve():
        try:
            connection, _ = listener.accept()
            with connection:
                connection.settimeout(5)
                handler(connection)
        except BaseException as error:
            errors.append(error)

    thread = threading.Thread(target=serve, daemon=True)
    thread.start()
    try:
        yield listener.getsockname()[1]
    finally:
        thread.join(6)
        listener.close()
        assert not thread.is_alive()
        if errors:
            raise errors[0]


def receive(connection, size):
    data = b""
    while len(data) < size:
        part = connection.recv(size - len(data))
        assert part, "connection closed"
        data += part
    return data


def receive_headers(connection):
    data = b""
    while not data.endswith(b"\r\n\r\n"):
        data += receive(connection, 1)
    return data.decode()


@pytest.mark.parametrize("scheme", ["socks5", "socks5h"])
def test_socks_proxy_auth_and_hostname_resolution(scheme):
    seen = []

    def serve(connection):
        version, length = receive(connection, 2)
        assert version == 5
        assert 2 in receive(connection, length)
        connection.sendall(b"\x05\x02")
        version, length = receive(connection, 2)
        assert version == 1
        assert receive(connection, length) == b"socks-user"
        assert receive(connection, receive(connection, 1)[0]) == b"socks-pass"
        connection.sendall(b"\x01\x00")
        version, command, reserved, kind = receive(connection, 4)
        assert (version, command, reserved) == (5, 1, 0)
        if kind == 3:
            host = receive(connection, receive(connection, 1)[0]).decode()
        else:
            assert kind == 1
            host = ".".join(str(part) for part in receive(connection, 4))
        port = int.from_bytes(receive(connection, 2), "big")
        seen.append((host, port))
        connection.sendall(b"\x05\x00\x00\x01\x7f\x00\x00\x01\x00\x50")
        assert receive_headers(connection).startswith("GET /test HTTP/")
        connection.sendall(
            b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nsocks"
        )

    host = "remote.invalid" if scheme == "socks5h" else "127.0.0.1"
    with socket_server(serve) as port:
        with brimp.Session(
            proxy=f"{scheme}://127.0.0.1:{port}",
            proxy_auth=("socks-user", "socks-pass"),
            plugins=[],
        ) as session:
            assert session.get(f"http://{host}:8765/test").text == "socks"
    assert seen == [(host, 8765)]


def test_websocket_route_proxy_and_filter(endpoints):
    import hashlib

    (origin, _), _, _ = endpoints
    requests = []

    def serve(connection):
        head = receive_headers(connection)
        requests.append(head)
        assert head.startswith("CONNECT remote.invalid:80 HTTP/")
        connection.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        head = receive_headers(connection)
        assert head.startswith("GET /echo HTTP/")
        headers = dict(line.split(": ", 1) for line in head.split("\r\n")[1:] if line)
        assert any(key.lower() == "sec-websocket-key" for key in headers), head
        key = next(value for key, value in headers.items() if key.lower() == "sec-websocket-key")
        accept = base64.b64encode(
            hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
        ).decode()
        connection.sendall(
            f"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n".encode()
        )
        connection.sendall(b"\x81\x05hello")
        # The client closes after receiving the message.
        connection.recv(1024)

    with socket_server(serve) as port:
        with brimp.Session(plugins=[]) as session:
            observed = []

            def policy(route):
                observed.append(route.request["resource_type"])
                route.continue_request(proxy=f"http://127.0.0.1:{port}")

            session.route("ws://*", policy)
            page = session.get(origin)
            assert (
                page.evaluate(
                    "new Promise((resolve, reject) => {const s = new WebSocket('ws://remote.invalid/echo'); s.onmessage = e => {s.close(); resolve(e.data)}; s.onerror = () => reject(new Error('failed'));})"
                )
                == "hello"
            )
            assert observed == ["websocket"]
            session.route("ws://*", lambda route: route.fail("blocked socket"))
            assert (
                page.evaluate(
                    "new Promise(resolve => {const s = new WebSocket('ws://remote.invalid/blocked'); s.onerror = () => resolve('blocked');})"
                )
                == "blocked"
            )
    assert len(requests) == 1
    assert requests[0].startswith("CONNECT remote.invalid:80 HTTP/")


def test_routes_override_invalid_environment_before_transport(endpoints, monkeypatch):
    (origin, _), _, _ = endpoints
    monkeypatch.setenv("http_proxy", "ftp://unsupported.invalid")
    with brimp.Session(plugins=[]) as session:
        session.route("*", lambda route: route.continue_request(proxy=""))
        assert session.get(origin).json()["server"] == "origin"
        session.route("*", lambda route: route.fail("policy blocked"))
        with pytest.raises(brimp.TransportError, match="policy blocked"):
            session.get(origin)
        session.route("*", lambda route: route.fulfill(body="offline fixture"))
        assert session.get(origin).text == "offline fixture"
