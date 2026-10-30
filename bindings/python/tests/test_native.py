from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading
import json
from pathlib import Path
import pytest
from brimp import Session, JavaScriptError, StaleHandle, Closed


HTML = b"""<!doctype html><main><b>old</b></main><script>
globalThis.beforeLater = document.querySelector('#later') === null;
document.querySelector('b').textContent = 'script';</script><p id="later">later</p>"""


@pytest.fixture(scope="module")
def server():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if (
                self.path == "/auth-challenge"
                and self.headers.get("Authorization") != "Basic dXNlcjpwYXNz"
            ):
                self.send_response(401)
                self.send_header("WWW-Authenticate", 'Basic realm="local-test"')
                self.end_headers()
                self.wfile.write(b"credentials required")
                return
            if self.path == "/over-interception-limit":
                self.send_response(200)
                self.end_headers()
                try:
                    for _ in range(33):
                        self.wfile.write(b"x" * (1024 * 1024))
                except (BrokenPipeError, ConnectionResetError):
                    pass
                return
            if self.path == "/large":
                self.send_response(200)
                self.end_headers()
                try:
                    self.wfile.write(bytes(range(251)) * 8192)
                except (BrokenPipeError, ConnectionResetError):
                    pass
                return
            if self.path == "/redirect":
                self.send_response(302)
                self.send_header("Location", "/")
                self.end_headers()
                return
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            self.wfile.write(
                json.dumps(dict(self.headers)).encode() if self.path == "/headers" else HTML
            )

        def do_POST(self):
            data = self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, *_):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{server.server_port}"
    server.shutdown()
    server.server_close()
    thread.join()


def test_request_script_dom_and_handles(server):
    with Session() as session:
        with session.get(server + "/redirect") as page:
            assert page.status_code == 200
            assert page.content == HTML
            assert len(page.response.history) == 1
            assert page.evaluate("beforeLater") is True
            with page.query_selector("main") as root:
                node = root.query_selector("b")
                assert node.text_content() == "script"
                node.remove()
                node.set_text_content("detached")
                assert node.text_content() == "detached"
                assert page.query_selector("b") is None
                page.get(server)
                with pytest.raises(StaleHandle):
                    node.text_content()
            with page.evaluate_handle("({a: 1})") as handle:
                assert handle.property_names() == ["a"]
                with handle.call("(value, n) => value.a + n", 41) as result:
                    assert result.json_value() == 42
            with pytest.raises(StaleHandle):
                handle.json_value()
        with pytest.raises(Closed):
            page.evaluate("42")


def test_promises_bytes_and_errors(server):
    with Session() as session:
        page = session.post(server, body=b"\x00\xffhello")
        assert page.content == b"\x00\xffhello"
        assert page.evaluate("new Promise(resolve => setTimeout(() => resolve(42), 5))") == 42
        with pytest.raises(JavaScriptError, match="boom"):
            page.evaluate("Promise.reject(new Error('boom'))")
        assert page.evaluate("42") == 42


def test_concurrent_pages_cookie_isolation_and_reuse(server):
    with Session() as first, Session() as second:
        first.set_cookie(server, "secret", "one")
        with ThreadPoolExecutor(3) as pool:
            a, b, c = [
                f.result() for f in [pool.submit(s.get, server) for s in (first, first, second)]
            ]
        assert a.evaluate("document.cookie") == b.evaluate("document.cookie") == "secret=one"
        assert c.evaluate("document.cookie") == ""
        a.evaluate("globalThis.onlyA = 1")
        assert b.evaluate("typeof onlyA") == "undefined"
        a.get(server)
        assert a.evaluate("typeof onlyA") == "undefined"
        first.close()
        assert c.evaluate("42") == 42


def test_close_interrupts_pending_evaluation_without_gil_deadlock():
    import time

    with Session(timeout=2) as session:
        page = session.new_page()
        with ThreadPoolExecutor(1) as pool:
            pending = pool.submit(page.evaluate, "new Promise(() => {})")
            time.sleep(0.02)
            page.close()
            with pytest.raises(Closed):
                pending.result(timeout=1)


def test_request_defaults_overrides_and_auth(server):
    with Session(
        headers={"X-Default": "default", "X-Keep": "keep"}, auth=("user", "pass")
    ) as session:
        page = session.get(server + "/headers", headers={"X-Default": "override"})
        headers = {k.lower(): v for k, v in page.json().items()}
        assert headers["x-default"] == "override"
        assert headers["x-keep"] == "keep"
        assert headers["authorization"] == "Basic dXNlcjpwYXNz"


def test_network_routes_events_reentry_and_controls(server):
    from brimp import RuntimeError, Reentry

    with Session() as session:
        page = session.new_page()
        observed = []
        loaded = threading.Event()
        for name in ("request", "response", "request_finished", "console", "mutation"):
            page.on(name, observed.append)
        page.on("lifecycle", lambda event: loaded.set() if event["name"] == "load" else None)

        def request(route):
            with pytest.raises(Reentry):
                page.evaluate("1")
            route.continue_request(headers={"X-Intercepted": "yes"})

        off = page.route("*", request)
        page.get(server + "/headers")
        assert {key.lower(): value for key, value in page.json().items()}["x-intercepted"] == "yes"
        assert loaded.wait(2)
        assert [record["event"] for record in observed[:3]] == [
            "request",
            "response",
            "request_finished",
        ]
        assert page.response_body(observed[0]["request"]["id"]) == page.content
        off()

        def response(route):
            assert route.body()
            route.continue_request(
                status=201, body=b"changed", headers={"Content-Type": "text/plain"}
            )

        off = page.route("*", response, stage="response")
        page.get(server)
        assert page.status_code == 201 and page.content == b"changed"
        off()
        off = page.route("*", lambda route: None)
        with pytest.raises(RuntimeError, match="without completing"):
            page.get(server)
        off()
        page.configure_network(offline=True)
        with pytest.raises(RuntimeError, match="offline"):
            page.get(server)
        assert page.evaluate("navigator.onLine") is False


def test_stream_backpressure_identity_and_cancellation(server):
    import time
    from brimp import RuntimeError

    with Session(
        user_agent="Mozilla/5.0 (Test) Test/1 Safari/605.1.15", languages=["fr", "en"]
    ) as session:
        page = session.new_page()
        assert page.evaluate("[navigator.userAgent, navigator.language]") == [
            "Mozilla/5.0 (Test) Test/1 Safari/605.1.15",
            "fr",
        ]
        page.set_identity(user_agent="Mozilla/5.0 (Test) Test/2 Safari/605.1.15")
        with page.stream("GET", server + "/headers") as response:
            assert {k.lower(): v for k, v in json.loads(b"".join(response)).items()}[
                "user-agent"
            ] == "Mozilla/5.0 (Test) Test/2 Safari/605.1.15"
        with page.stream("GET", server + "/large") as response:
            time.sleep(0.05)
            assert page.evaluate("6*7") == 42
            assert b"".join(response) == bytes(range(251)) * 8192
        with page.stream("GET", server + "/redirect") as response:
            assert b"".join(response) == HTML
        response = page.stream("GET", server + "/large")
        page.close()
        with pytest.raises(RuntimeError, match="cancelled"):
            next(response)


def test_response_interception_enforces_transport_body_limit(server):
    from brimp import RuntimeError

    with Session() as session:
        page = session.new_page()
        called = []
        page.route("*", lambda route: called.append(route), stage="response")
        for request in (
            lambda: page.get(server + "/over-interception-limit"),
            lambda: page.stream("GET", server + "/over-interception-limit"),
        ):
            with pytest.raises(RuntimeError, match=r"33554432|32 MiB|body.*limit"):
                request()
        assert called == []  # Oversized bodies never reach the host handler.


def test_authentication_callback_retries_requests_and_streams(server):
    with Session() as session:
        page = session.new_page()
        challenges = []

        def authenticate(route):
            assert route.stage == "auth"
            challenges.append(route.auth_challenge)
            route.authenticate("user", "pass")

        page.route("*", authenticate, stage="auth")
        page.get(server + "/auth-challenge")
        assert page.status_code == 200
        with page.stream("GET", server + "/auth-challenge") as response:
            assert response.status_code == 200
            assert b"".join(response) == HTML
        assert len(challenges) == 2
        assert all("local-test" in challenge for challenge in challenges)


def test_profile_option_drives_environment_values():
    profile = {"values": {"identity": {}, "screen": {}}}
    profile["values"]["identity"]["languages"] = ["fr-CA", "fr"]
    profile["values"]["identity"]["locale"] = "fr-CA"
    profile["values"]["identity"]["timezone"] = "America/New_York"
    profile["values"]["screen"]["width"] = 1920
    profile["values"]["screen"]["avail_width"] = 1920
    with Session(profile=profile) as session:
        page = session.new_page()
        assert page.evaluate(
            "[navigator.language, navigator.languages, screen.width, new Intl.DateTimeFormat().resolvedOptions().timeZone]"
        ) == ["fr-CA", ["fr-CA", "fr"], 1920, "America/New_York"]
        assert page.evaluate("new Date('2026-01-15T12:00:00Z').getTimezoneOffset()") == 300
