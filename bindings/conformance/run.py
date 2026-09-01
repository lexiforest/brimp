#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HTML = b"""<!doctype html><title>Bindings</title><main>Hello bindings</main>
<button id='submit'>Submit</button><input id='name'><button id='tap'>Tap</button>
<script>
globalThis.inputEvents = [];
for (const target of document.querySelectorAll('button,input')) {
  for (const type of ['pointerover','pointerenter','pointermove','pointerdown','mousedown','focus','keydown','input','keyup','touchstart','touchend','pointerup','mouseup','click']) {
    target.addEventListener(type, event => inputEvents.push({id: target.id, type, trusted: event.isTrusted, pointerType: event.pointerType || ''}));
  }
}
</script>"""

class Handler(BaseHTTPRequestHandler):
    def handle_request(self):
        if self.path.startswith("/hang"):
            time.sleep(5)
            return
        length = int(self.headers.get("Content-Length", 0))
        request_body = self.rfile.read(length)
        if self.path.startswith("/redirect"):
            self.send_response(302)
            self.send_header("Location", "/final")
            self.end_headers()
            return
        if self.path.startswith("/echo"):
            body = json.dumps({
                "method": self.command,
                "body": request_body.decode(),
                "contentType": self.headers.get("Content-Type"),
            }).encode()
            status = 200
            content_type = "application/json; charset=utf-8"
        elif self.path.startswith("/inspect"):
            body = json.dumps({
                "header": self.headers.get("X-Binding"),
                "cookie": self.headers.get("Cookie"),
            }).encode()
            status = 200
            content_type = "application/json; charset=utf-8"
        elif self.path.startswith("/missing"):
            body = HTML
            status = 404
            content_type = "text/html; charset=utf-8"
        else:
            body = HTML
            status = 200
            content_type = "text/html; charset=utf-8"
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Set-Cookie", "server=ready; Path=/")
        self.end_headers()
        self.wfile.write(body)
    do_GET = handle_request
    do_POST = handle_request
    do_PUT = handle_request
    do_PATCH = handle_request
    do_DELETE = handle_request
    do_OPTIONS = handle_request
    def log_message(self, *_): pass

def run(command, env=None):
    result = subprocess.run(command, text=True, capture_output=True, env=env, check=False)
    if result.returncode:
        raise RuntimeError(f"{' '.join(command)} failed ({result.returncode}):\n{result.stderr}")
    return json.loads(result.stdout)

def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: run.py PYTHON_PACKAGE_DIR|- NODE_PACKAGE_DIR")
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
    url = f"http://127.0.0.1:{server.server_port}"
    env = os.environ.copy()
    if sys.argv[1] == "-":
        env.pop("PYTHONPATH", None)
    else:
        env["PYTHONPATH"] = sys.argv[1]
    python_executable = env.get("BRIMP_PYTHON", sys.executable)
    try:
        python = run([python_executable, os.path.join(os.path.dirname(__file__), "scenario.py"), url], env)
        node = run(["node", os.path.join(os.path.dirname(__file__), "scenario.mjs"), url, sys.argv[2]])
    finally:
        server.shutdown(); thread.join()
    if node.pop("cancelled", None) is not True:
        raise RuntimeError("Node AbortSignal did not cancel navigation")
    if python != node:
        raise RuntimeError(f"binding results differ:\npython={python}\nnode={node}")
    print(json.dumps(python, sort_keys=True))

if __name__ == "__main__": main()
