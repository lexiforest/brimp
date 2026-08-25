#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HTML = b"<!doctype html><title>Bindings</title><main>Hello bindings</main>"

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/hang":
            time.sleep(5)
            return
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(HTML)))
        self.end_headers()
        self.wfile.write(HTML)
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
    if python != node:
        raise RuntimeError(f"binding results differ:\npython={python}\nnode={node}")
    print(json.dumps(python, sort_keys=True))

if __name__ == "__main__": main()
