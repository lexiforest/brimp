"""Local HTTP fixture for Brimp tutorials. Run from the repository root."""

import argparse
import json
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

HTML = b"""<!doctype html><html lang="en"><head><title>Brimp tutorial</title></head>
<body><article><h1>Waiting for JavaScript</h1>
<p>This article is served by your local tutorial server. Brimp executes its script
and lets you read the updated document, change elements, and extract Markdown.</p>
<p>Sessions retain cookies and connections. Each page owns a JavaScript realm and
DOM. Close the session when your work is done to release its pages and handles.</p>
</article><form><input id="name" name="name"><button type="submit">Greet</button>
<output id="result"></output></form><a href="/article">Read again</a>
<script>document.querySelector('h1').textContent = 'Hello from JavaScript';
document.querySelector('form').addEventListener('submit', event => {
 event.preventDefault();
 document.querySelector('#result').textContent = 'Hello, ' + document.querySelector('#name').value;
});</script></body></html>"""


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        url = urlsplit(self.path)
        if url.path == "/slow":
            time.sleep(3)
        if url.path == "/api/status":
            return self.send(200, "application/json", json.dumps({"ok": True}).encode())
        if url.path == "/echo":
            return self.send(
                200,
                "application/json",
                json.dumps(
                    {
                        "method": "GET",
                        "query": parse_qs(url.query),
                        "cookie": self.headers.get("Cookie", ""),
                    }
                ).encode(),
            )
        if url.path == "/download":
            return self.send(200, "application/octet-stream", b"Brimp stream\n" * 4096)
        self.send(200, "text/html; charset=utf-8", HTML)

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
        self.send(
            200, "application/json", json.dumps({"method": "POST", "body": body.decode()}).encode()
        )

    def send(self, status, content_type, body):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        try:
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass  # Cancellation is an intentional tutorial exercise.

    def log_message(self, *_):
        pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8765)
    args = parser.parse_args()
    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    print(f"Tutorial server: http://127.0.0.1:{server.server_port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
