import json
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import brimp


class Handler(BaseHTTPRequestHandler):
    observed = []

    def do_GET(self):
        type(self).observed.append((self.path, self.headers.get("X-Test"), self.headers.get("Cookie")))
        if self.path.startswith("/missing"):
            status = 404
            content_type = "text/html; charset=utf-8"
            body = b"<main id='value'>raw</main><script>previousRealm = true; document.getElementById('value').textContent = 'rendered'</script>"
        elif self.path == "/next":
            status = 200
            content_type = "text/html"
            body = b"<main id='realm'></main><script>document.getElementById('realm').textContent = typeof previousRealm</script>"
        elif self.path == "/json":
            status = 200
            content_type = "application/json; charset=utf-8"
            body = b'{"answer":42}'
        else:
            status = 200
            content_type = "text/html"
            body = b"<title>Top level</title>"
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Set-Cookie", "server=ready; Path=/")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        return


class ApiTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.url = f"http://127.0.0.1:{cls.server.server_port}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.thread.join()

    def test_requests_style_response_session_state_and_clean_realms(self):
        with brimp.Session() as session:
            session.headers["X-Test"] = "session"
            session.cookies["manual"] = "yes"
            response = session.get(
                self.url + "/missing",
                params={"q": ["one", "two"]},
            )

            self.assertEqual(response.status_code, 404)
            self.assertEqual(response.reason, "Not Found")
            self.assertEqual(response.headers["CONTENT-TYPE"], "text/html; charset=utf-8")
            self.assertIn(">raw<", response.text)
            self.assertIn(">rendered<", response.html)
            self.assertFalse(response.ok)
            self.assertGreater(response.elapsed, 0)
            with self.assertRaises(brimp.HTTPError) as raised:
                response.raise_for_status()
            self.assertIs(raised.exception.response, response)
            self.assertEqual(session.cookies["server"], "ready")
            path, _, _ = Handler.observed[-1]
            self.assertEqual(path, "/missing?q=one&q=two")

            next_response = session.get(self.url + "/next")
            self.assertIn(">undefined<", next_response.html)
            _, header, cookie = Handler.observed[-1]
            self.assertEqual(header, "session")
            self.assertIn("manual=yes", cookie)
            self.assertIn("server=ready", cookie)

            data = session.get(self.url + "/json")
            self.assertEqual(data.json(), {"answer": 42})
            self.assertIsNone(data.html)

            with self.assertRaises(brimp.InvalidRequest):
                session.get(self.url, headers={"User-Agent": "incoherent"})

        with self.assertRaises(brimp.BrimpError):
            session.evaluate("document.title")

    def test_top_level_get_and_screenshot_path(self):
        response = brimp.get(self.url)
        self.assertEqual(response.status_code, 200)
        self.assertIn("Top level", response.html)

        with brimp.Session() as session, tempfile.TemporaryDirectory() as directory:
            session.get(self.url)
            path = Path(directory) / "page.png"
            content = session.screenshot(path)
            self.assertEqual(path.read_bytes(), content)
            self.assertTrue(content.startswith(b"\x89PNG\r\n\x1a\n"))

    def test_page_subsystems_are_opt_in(self):
        with brimp.Session() as session:
            self.assertEqual(
                session.evaluate(
                    "[typeof Worker, typeof WebSocket, typeof indexedDB, typeof navigator.storage, typeof HTMLCanvasElement.prototype.getContext, 'gpu' in navigator, typeof AudioContext]"
                ),
                ["undefined", "undefined", "undefined", "undefined", "undefined", False, "undefined"],
            )

        with tempfile.TemporaryDirectory() as directory:
            with brimp.Session(
                enable_worker=True,
                enable_streaming_networking=True,
                storage_path=directory,
            ) as session:
                self.assertEqual(
                    session.evaluate("typeof navigator.storage"),
                    "object",
                )


if __name__ == "__main__":
    unittest.main()
