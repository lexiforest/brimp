"""Measure the native Python request/script/query/edit/evaluate workflow."""

import json
import resource
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from brimp import Session

html = (
    "<!doctype html><main>"
    + "<p class='item'>text</p>" * 3000
    + "</main><script>globalThis.loaded = true;</script>"
).encode()


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(html)))
        self.end_headers()
        self.wfile.write(html)

    def log_message(self, *_):
        pass


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
start = time.perf_counter()
try:
    with Session() as session:
        page = session.new_page()
        for _ in range(20):
            page.get(f"http://127.0.0.1:{server.server_port}")
            assert page.evaluate("loaded") is True
            with page.query_selector("main > .item") as node:
                node.set_text_content("edited")
                assert node.text_content() == "edited"
            assert page.evaluate("document.querySelectorAll('.item').length") == 3000
    print(
        json.dumps(
            {
                "iterations": 20,
                "elapsed_ms": (time.perf_counter() - start) * 1000,
                "max_rss_platform_units": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
            }
        )
    )
finally:
    server.shutdown()
    server.server_close()
    thread.join()
