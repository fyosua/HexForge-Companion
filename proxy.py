"""
HexForge Companion — Mock API Proxy for browser preview.

Run this alongside the Tauri app to let the browser frontend
test the full search/stats flow without needing the Tauri backend.
Starts on port 1421.

Usage:
  python3 proxy.py
"""

import json
import http.server
import urllib.parse

HOST = "0.0.0.0"
PORT = 1421

MOCK_PUUID = "S7vF9kG2hJ5mN8qR3tW1xZ4cB6yA0dE8fL7pO2iU9sK4jH5gF3vB1nM6xC0zR"

MOCK_PLAYER = {
    "puuid": MOCK_PUUID,
    "game_name": "HexTactician",
    "tag_line": "KR1",
    "summoner_level": 482,
}

MOCK_MATCHES = [
    {"match_id": "KR_...1-abc",  "game_datetime": 1717858800000, "placement": 1,  "game_version": "14.11"},
    {"match_id": "KR_...2-def", "game_datetime": 1717855200000, "placement": 3,  "game_version": "14.11"},
    {"match_id": "KR_...3-ghi", "game_datetime": 1717851600000, "placement": 5,  "game_version": "14.10"},
    {"match_id": "KR_...4-jkl", "game_datetime": 1717848000000, "placement": 2,  "game_version": "14.10"},
    {"match_id": "KR_...5-mno", "game_datetime": 1717844400000, "placement": 4,  "game_version": "14.10"},
]

MOCK_STATS = {
    "total_games": 5,
    "avg_placement": 3.0,
    "wins": 1,
    "top4": 3,
    "win_rate_pct": 20.0,
}


class MockAPIHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == "/api/health":
            self._json(200, {"status": "ok", "mode": "mock"})
        else:
            self._json(404, {"error": "not found"})

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        content_len = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_len) if content_len else b"{}"
        try:
            data = json.loads(body)
        except json.JSONDecodeError:
            data = {}

        if parsed.path == "/api/resolve-player":
            self._json(200, MOCK_PLAYER)

        elif parsed.path == "/api/get-match-history":
            limit = data.get("limit", 5)
            self._json(200, MOCK_MATCHES[:limit])

        elif parsed.path == "/api/get-player-stats":
            self._json(200, MOCK_STATS)

        else:
            self._json(404, {"error": "not found"})

    def _json(self, status, obj):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()
        self.wfile.write(json.dumps(obj).encode())

    def do_OPTIONS(self):
        self._json(200, {})

    def log_message(self, format, *args):
        print(f"[proxy] {args[0]} {args[1]} {args[2]}")


if __name__ == "__main__":
    server = http.server.HTTPServer((HOST, PORT), MockAPIHandler)
    print(f"[HexForge Proxy] Mock API running on http://{HOST}:{PORT}")
    print(f"[HexForge Proxy] Endpoints: /api/health, /api/resolve-player, /api/get-match-history, /api/get-player-stats")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[HexForge Proxy] Shutdown")
        server.server_close()