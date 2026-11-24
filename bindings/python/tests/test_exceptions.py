"""Public exception contracts exercised through actual native request failures."""

import asyncio
import json
import pickle
import socket
import ssl
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

import brimp
from test_proxies import receive_headers, socket_server


@pytest.fixture(scope="module")
def server():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/slow":
                time.sleep(0.3)
            status = 404 if self.path == "/missing" else 200
            body = b"not json"
            if self.path == "/redirect":
                self.send_response(302)
                self.send_header("Location", "/redirect")
                self.end_headers()
                return
            if self.path == "/loop":
                body = b"<script>while(true) {}</script>"
            self.send_response(status)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def log_message(self, *_):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{httpd.server_port}"
    httpd.shutdown()
    httpd.server_close()
    thread.join()


def test_hierarchy_and_public_namespace():
    assert brimp.RequestsError is brimp.RequestException
    assert not hasattr(brimp, "CurlError")
    for name in [
        "ConnectionError",
        "DNSError",
        "ProxyError",
        "SSLError",
        "CertificateVerifyError",
        "HTTPError",
        "IncompleteRead",
        "RequestTimeout",
        "TooManyRedirects",
        "SessionClosed",
        "ImpersonateError",
        "InvalidURL",
        "MissingSchema",
        "InvalidSchema",
        "InvalidProxyURL",
        "InvalidHeader",
        "InterfaceError",
        "JSONDecodeError",
    ]:
        cls = getattr(brimp, name)
        assert cls is getattr(brimp.exceptions, name)
        assert issubclass(cls, (brimp.RequestException,))
        assert issubclass(cls, brimp.BrimpError)
        assert issubclass(cls, OSError)
    assert issubclass(brimp.CertificateVerifyError, brimp.SSLError)
    assert issubclass(brimp.DNSError, brimp.ConnectionError)
    assert issubclass(brimp.IncompleteRead, brimp.HTTPError)
    assert issubclass(brimp.InvalidURL, brimp.InvalidInput)
    assert issubclass(brimp.InvalidURL, ValueError)
    assert issubclass(brimp.SessionClosed, brimp.Closed)
    assert issubclass(brimp.RequestTimeout, brimp.Timeout)
    for cls in [
        brimp.JavaScriptError,
        brimp.StaleHandle,
        brimp.Reentry,
        brimp.Timeout,
        brimp.Closed,
        brimp.InvalidInput,
    ]:
        assert issubclass(cls, brimp.BrimpError)
        assert not issubclass(cls, brimp.RequestException)
    error = pickle.loads(pickle.dumps(brimp.ProxyError("failed", 97)))
    assert error.code == 97 and error.response is None and error.page is None


def test_refused_connection_preserves_curl_code():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
        listener.close()
        with brimp.Session(trust_env=False, plugins=[]) as session:
            with pytest.raises(brimp.ConnectionError) as raised:
                session.get(f"http://127.0.0.1:{port}")
    assert raised.value.code == 7
    assert isinstance(raised.value, brimp.TransportError)
    assert raised.value.response is None


def test_dns_failure_preserves_curl_code():
    with brimp.Session(trust_env=False, plugins=[], timeout=5) as session:
        with pytest.raises(brimp.DNSError) as raised:
            # Oversized DNS labels fail locally, without depending on an external resolver.
            session.get("http://" + "x" * 64 + ".invalid/")
    assert raised.value.code == 6


def test_connect_proxy_rejection_is_classified_without_parsing_messages():
    requests = []

    class Reject(BaseHTTPRequestHandler):
        def reject(self):
            requests.append(self.command)
            self.send_response(403)
            self.send_header("Content-Length", "0")
            self.end_headers()

        do_CONNECT = do_GET = reject

        def log_message(self, *_):
            pass

    # Handle HTTP/3's CONNECT-UDP attempt and its ordinary CONNECT fallback.
    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Reject)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    try:
        with brimp.Session(
            proxy=f"http://127.0.0.1:{httpd.server_port}", trust_env=False, plugins=[]
        ) as session:
            with pytest.raises(brimp.ProxyError) as raised:
                session.get("https://example.invalid/")
        assert raised.value.code != 0
        assert raised.value.response is None
        assert "CONNECT" in requests
    finally:
        httpd.shutdown()
        httpd.server_close()
        thread.join()


@pytest.mark.parametrize("proxy", [False, True])
def test_untrusted_tls_certificate(proxy):
    fixtures = Path(__file__).with_name("fixtures")
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(fixtures / "localhost-cert.pem", fixtures / "localhost-key.pem")
    httpd = ThreadingHTTPServer(("127.0.0.1", 0), BaseHTTPRequestHandler)
    httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    try:
        port = httpd.server_port
        options = {"proxy": f"https://127.0.0.1:{port}"} if proxy else {}
        url = "http://example.invalid/" if proxy else f"https://127.0.0.1:{port}/"
        with brimp.Session(trust_env=False, plugins=[], **options) as session:
            with pytest.raises(brimp.CertificateVerifyError) as raised:
                session.get(url)
        # HTTP/3 fallback can retain connect (7) or receive (56) errors after
        # certificate verification fails; Windows CI also reports code 56.
        # The exception uses the native verification result without rewriting code.
        assert raised.value.code in {7, 56, 60}
        assert f"({raised.value.code})" in str(raised.value)
    finally:
        httpd.shutdown()
        httpd.server_close()
        thread.join()


@pytest.mark.parametrize("stream", [False, True])
def test_truncated_body_is_typed_for_buffered_and_streamed_requests(stream):
    def truncated(connection):
        receive_headers(connection)
        connection.sendall(
            b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nshort"
        )

    with socket_server(truncated) as port:
        with brimp.Session(trust_env=False, plugins=[]) as session:
            page = session.new_page()
            with pytest.raises(brimp.IncompleteRead) as raised:
                if stream:
                    with page.stream("GET", f"http://127.0.0.1:{port}/") as response:
                        b"".join(response)
                else:
                    page.get(f"http://127.0.0.1:{port}/")
    assert raised.value.code == 18


@pytest.mark.parametrize("stream", [False, True])
def test_redirect_limit_and_request_deadline(server, stream):
    with brimp.Session(trust_env=False, plugins=[]) as session:
        page = session.new_page()
        with pytest.raises(brimp.TooManyRedirects) as raised:
            if stream:
                page.stream("GET", server + "/redirect", max_redirects=1)
            else:
                page.get(server + "/redirect", max_redirects=1)
        assert raised.value.code == 0
        with pytest.raises(brimp.RequestTimeout) as raised:
            if stream:
                page.stream("GET", server + "/slow", timeout=0.05)
            else:
                page.get(server + "/slow", timeout=0.05)
        assert isinstance(raised.value, brimp.Timeout)
        assert isinstance(raised.value, brimp.RequestException)


def test_runtime_timeout_stays_distinct(server):
    with brimp.Session(trust_env=False, plugins=[]) as session:
        with pytest.raises(brimp.Timeout) as raised:
            session.get(server + "/loop", timeout=0.1)
        assert not isinstance(raised.value, brimp.RequestException)
        page = session.new_page()
        with pytest.raises(brimp.JavaScriptError) as raised:
            page.evaluate("throw new Error('runtime')")
        assert not isinstance(raised.value, brimp.RequestException)


@pytest.mark.parametrize(
    "url,kind",
    [
        ("relative/path", brimp.MissingSchema),
        ("ftp://example.com", brimp.InvalidSchema),
        ("http://[", brimp.InvalidURL),
        ("http://example.com:bad", brimp.InvalidURL),
        ("http://", brimp.InvalidURL),
    ],
)
def test_invalid_urls(server, url, kind):
    with brimp.Session(trust_env=False, plugins=[]) as session:
        with pytest.raises(kind) as raised:
            session.get(url)
        assert isinstance(raised.value, ValueError)
        assert isinstance(raised.value, brimp.InvalidInput)
        assert raised.value.code == 0


def test_invalid_headers_proxy_and_session_closed(server):
    with pytest.raises(brimp.InvalidHeader):
        brimp.Session(headers={"X-Test": "bad\r\nvalue"})
    with pytest.raises(brimp.InvalidProxyURL) as raised:
        brimp.Session(proxy="http://user:secret@[")
    assert "secret" not in str(raised.value)
    with brimp.Session(plugins=[]) as session:
        with pytest.raises(brimp.InvalidHeader):
            session.get(server, headers={"bad name": "value"})
    with pytest.raises(brimp.SessionClosed):
        session.get(server)


def test_impersonation_failure_is_typed():
    with pytest.raises(brimp.ImpersonateError):
        brimp.Session(profile={"values": {"network": {"impersonation": "not-a-profile"}}})


def test_http_and_json_errors_keep_live_page_and_metadata(server):
    with brimp.Session(trust_env=False, plugins=[]) as session:
        page = session.get(server + "/missing")
        with pytest.raises(brimp.HTTPError) as raised:
            page.raise_for_status()
        error = raised.value
        assert error.response is error.page is page
        assert error.status_code == 404 and error.code == 0
        assert error.url == server + "/missing"
        page.get(server)
        assert error.status_code == 404 and error.url.endswith("/missing")
        with pytest.raises(brimp.JSONDecodeError) as raised:
            page.json()
        error = raised.value
        assert isinstance(error, json.JSONDecodeError)
        assert error.response is error.page is page
        assert error.pos == 0 and error.doc == "not json"
        assert isinstance(error.__cause__, json.JSONDecodeError)
        bare = brimp.JSONDecodeError(error.msg, error.doc, error.pos)
        restored = pickle.loads(pickle.dumps(bare))
        assert restored.pos == error.pos and restored.doc == error.doc


def test_async_uses_same_errors_and_async_page_attachment(server):
    async def run():
        async with brimp.AsyncSession(trust_env=False, plugins=[]) as session:
            page = await session.get(server + "/missing")
            with pytest.raises(brimp.HTTPError) as raised:
                page.raise_for_status()
            assert raised.value.response is raised.value.page is page
            with pytest.raises(brimp.JSONDecodeError) as raised:
                page.json()
            assert raised.value.response is page
            with pytest.raises(brimp.RequestTimeout):
                await session.get(server + "/slow", timeout=0.05)
        with pytest.raises(brimp.SessionClosed):
            await session.get(server)

    asyncio.run(run())


def test_bad_content_encoding_preserves_http_error_code():
    def encoded(connection):
        receive_headers(connection)
        connection.sendall(
            b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: 7\r\nConnection: close\r\n\r\ninvalid"
        )

    with socket_server(encoded) as port:
        with brimp.Session(trust_env=False, plugins=[]) as session:
            with pytest.raises(brimp.HTTPError) as raised:
                session.get(f"http://127.0.0.1:{port}/")
    assert raised.value.code == 61
