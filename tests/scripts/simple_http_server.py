#!/usr/bin/env python3

from http.server import SimpleHTTPRequestHandler
import socketserver
import sys


class HTTPRequestHandler(SimpleHTTPRequestHandler):
    def do_PUT(self):
        length = int(self.headers["Content-Length"])

        path = self.translate_path(self.path)
        with open(path, "wb") as dst:
            dst.write(self.rfile.read(length))

        self.send_response(200)
        self.end_headers()


if __name__ == '__main__':
    if len(sys.argv) > 1:
        port = int(sys.argv[1])
    else:
        port = 6789
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("localhost", port),
                                HTTPRequestHandler) as httpd:
        print("serving at port", port)
        httpd.serve_forever()
