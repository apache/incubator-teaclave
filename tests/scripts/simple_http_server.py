import SimpleHTTPServer


class HTTPRequestHandler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    def do_PUT(self):
        length = int(self.headers["Content-Length"])

        path = self.translate_path(self.path)
        with open(path, "wb") as dst:
            dst.write(self.rfile.read(length))

        self.send_response(200)
        self.end_headers()


if __name__ == '__main__':
    SimpleHTTPServer.test(HandlerClass=HTTPRequestHandler)
