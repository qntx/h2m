"""Minimal mock SearXNG returning a static JSON payload for smoke tests."""

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

PAYLOAD = {
    "results": [
        {
            "url": "https://rust-lang.org",
            "title": "Rust Programming Language",
            "content": "A language empowering everyone to build reliable and efficient software.",
            "engine": "google",
            "category": "general",
        },
        {
            "url": "https://doc.rust-lang.org",
            "title": "The Rust Reference",
            "content": "This book is the primary reference for the Rust programming language.",
            "engine": "bing",
            "category": "general",
        },
        {
            "url": "https://news.rust-lang.org",
            "title": "Rust Blog",
            "content": "Official blog of the Rust Programming Language.",
            "engine": "google",
            "category": "news",
            "publishedDate": "2025-12-01",
        },
    ]
}


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):  # noqa: D401
        """Silence noisy default logging."""

    def do_GET(self):  # noqa: N802
        body = json.dumps(PAYLOAD).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main() -> None:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8888
    server = HTTPServer(("127.0.0.1", port), Handler)
    print(f"mock SearXNG listening on http://127.0.0.1:{port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()


if __name__ == "__main__":
    main()
