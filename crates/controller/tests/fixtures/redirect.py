#!/usr/bin/env python3

from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from threading import Event


never_respond = Event()


class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/redirect':
            self.send_response(302)
            self.send_header('Location', '/lifecycle.html')
            self.end_headers()
            return
        if self.path == '/failure':
            self.connection.shutdown(2)
            self.connection.close()
            return
        if self.path == '/never-responds':
            never_respond.wait()
            return
        super().do_GET()


if __name__ == '__main__':
    ThreadingHTTPServer(('127.0.0.1', 8000), Handler).serve_forever()
