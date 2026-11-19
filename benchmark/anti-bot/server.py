"""Local stand-in for the sites the pyv8c examples target.

The examples spend their time in one loop: navigate, let the page's challenge code
run, watch for the cookie it sets, then call the site's API with that cookie. This
server reproduces that protocol locally so the client-side half can be exercised
without network access. It is *not* an Akamai/DataDome/PX challenge and cannot tell
anyone whether a real one would pass; it only answers whether the runtime can run
the same flow.
"""
from __future__ import annotations

import http.server
import json
import socketserver
import sys
import threading

PAGE = """<!doctype html><html><head><meta charset=utf-8><title>challenge stand-in</title>
<link rel="stylesheet" href="/theme.css">
</head><body>
<h1>Are you a person or a robot?</h1>
<p id="copy">press and hold</p>
<ul><li>one<li>two</ul>
<img id="logo" alt="">
<span id="empty"></span>
<script>
// The signals a sensor collects before it posts its payload.
window.__signals = null;
window.__collected = false;
const measure = () => {
  const canvas = document.createElement('canvas');
  canvas.width = 64; canvas.height = 32;
  const context = canvas.getContext('2d');
  if (context) { context.textBaseline = 'top'; context.fillStyle = '#123456'; context.fillText('challenge', 2, 2); }
  const gl = document.createElement('canvas').getContext('webgl');
  return {
    userAgent: navigator.userAgent,
    languages: navigator.languages,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    hardwareConcurrency: navigator.hardwareConcurrency,
    devicePixelRatio: devicePixelRatio,
    screen: [screen.width, screen.height, screen.colorDepth],
    canvas: canvas.toDataURL().slice(-32),
    webgl: gl ? [gl.getParameter(37445), gl.getParameter(37446), gl.getParameter(34921)] : null,
    rectIsDomRect: (() => { const r = document.body.getBoundingClientRect(); return r instanceof DOMRect; })(),
    pointerMoves: 0,
  };
};
addEventListener('pointermove', () => { if (window.__signals) window.__signals.pointerMoves += 1; });
window.collect = () => {
  window.__signals = measure();
  return fetch('/collect', { method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify(window.__signals) }).then((response) => response.status);
};
</script>
</body></html>
"""

THEME = "p#copy { color: rgb(1, 2, 3); }"


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *arguments):  # silence per-request logging
        pass

    def _send(self, status, body=b"", content_type="text/html; charset=utf-8", headers=()):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        for name, value in headers:
            self.send_header(name, value)
        self.end_headers()
        if body:
            self.wfile.write(body)

    def do_GET(self):
        if self.path.startswith("/theme.css"):
            return self._send(200, THEME.encode(), "text/css")
        if self.path.startswith("/api"):
            cookie = self.headers.get("Cookie", "")
            if "session=ok" in cookie:
                return self._send(200, json.dumps({"ok": True}).encode(), "application/json")
            return self._send(403, json.dumps({"ok": False}).encode(), "application/json")
        return self._send(200, PAGE.encode())

    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(length)
        if self.path.startswith("/collect"):
            payload = json.loads(body or b"{}")
            print(json.dumps({"collected": sorted(payload)}), flush=True)
            return self._send(204, b"", "text/plain", [("Set-Cookie", "session=ok; Path=/; HttpOnly")])
        return self._send(404)


class Server(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def serve(port=0):
    server = Server(("127.0.0.1", port), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, server.server_address[1]


if __name__ == "__main__":
    server, port = serve(int(sys.argv[1]) if len(sys.argv) > 1 else 0)
    print(port, flush=True)
    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        server.shutdown()
