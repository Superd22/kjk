"""Saves the nose page's answers. The page POSTs JSON to http://127.0.0.1:8766/save/<name>, and this writes it to
likeness-fit-data/sliders/answers/<name>.json, with a copy stamped with the time. Run: python3 -I answers_server.py (no packages)."""
import json, os, re, time
from http.server import BaseHTTPRequestHandler, HTTPServer

OUT = os.path.expanduser("~/.claude/tools/likeness-fit-data/sliders/answers")


class Handler(BaseHTTPRequestHandler):
    def cors(self):
        self.send_header("Access-Control-Allow-Origin", "http://127.0.0.1:8765")
        self.send_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def do_OPTIONS(self):
        self.send_response(204)
        self.cors()
        self.end_headers()

    def do_GET(self):
        m = re.fullmatch(r"/load/([a-z_]+)", self.path)
        path = m and os.path.join(OUT, m.group(1) + ".json")
        body = open(path, "rb").read() if path and os.path.exists(path) else b"null"
        self.send_response(200)
        self.cors()
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        m = re.fullmatch(r"/save/([a-z_]+)", self.path)
        data = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        try:
            json.loads(data)
        except ValueError:
            m = None
        if not m:
            self.send_response(400)
            self.cors()
            self.end_headers()
            return
        os.makedirs(OUT, exist_ok=True)
        for name in (m.group(1) + ".json", m.group(1) + time.strftime("-%Y%m%d-%H%M%S.json")):
            open(os.path.join(OUT, name), "wb").write(data)
        self.send_response(200)
        self.cors()
        self.end_headers()
        self.wfile.write(b"saved")

    def log_message(self, *a):
        pass


HTTPServer(("127.0.0.1", 8766), Handler).serve_forever()
