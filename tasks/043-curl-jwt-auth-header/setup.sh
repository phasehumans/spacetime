#!/usr/bin/env bash
set -e

apt-get update -qq && apt-get install -y -qq python3 curl jq

cat <<'EOF' > /tmp/mock_api.py
import json
from http.server import HTTPServer, BaseHTTPRequestHandler

TOKEN = "mock-jwt-token-998877"

class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/auth':
            length = int(self.headers.get('content-length', 0))
            body = json.loads(self.rfile.read(length))
            if body.get('username') == 'admin' and body.get('password') == 'secretpassword':
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps({'token': TOKEN}).encode())
                return
        self.send_response(401)
        self.end_headers()

    def do_GET(self):
        if self.path == '/data':
            auth = self.headers.get('Authorization', '')
            if auth == f'Bearer {TOKEN}':
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps({'status': 'authorized', 'payload': 'spacetime_authenticated_ok'}).encode())
                return
        self.send_response(403)
        self.end_headers()

HTTPServer(('127.0.0.1', 8000), Handler).serve_forever()
EOF

python3 /tmp/mock_api.py &
sleep 0.5
rm -f /root/data.json
