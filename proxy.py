"""
HexForge Companion — Mock API Proxy for browser preview.

Run this alongside the Tauri app to let the browser frontend
test the full search/stats flow without needing the Tauri backend.
Starts on port 1421.

Usage:
  python3 proxy.py

Endpoints:
  GET  /api/health
  POST /api/resolve-player
  POST /api/get-match-history
  POST /api/get-player-stats
  POST /api/get-player-rank
  POST /api/get-active-game-status
  POST /api/refresh-matches
  POST /api/get-player-region
  POST /api/get-challenger-standings
  POST /api/get-grandmaster-standings
  POST /api/get-master-standings
  POST /api/get-platform-status
"""

import json
import http.server
import urllib.parse
import os
import glob

HOST = "0.0.0.0"
PORT = 1421

# Directory where built binaries are placed
DOWNLOAD_DIR = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "src-tauri",
    "target",
    "x86_64-pc-windows-gnu",
    "release",
)

MOCK_PUUID = "S7vF9kG2hJ5mN8qR3tW1xZ4cB6yA0dE8fL7pO2iU9sK4jH5gF3vB1nM6xC0zR"

MOCK_PLAYER = {
    "puuid": MOCK_PUUID,
    "game_name": "HexTactician",
    "tag_line": "KR1",
    "summoner_level": 482,
    "summoner_id": "jX8mN2pQ5rT7vB9kL1cH3wF6yA0sD4gE",
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

MOCK_RANK = [
    {
        "tier": "DIAMOND",
        "rank": "II",
        "league_points": 43,
        "wins": 87,
        "losses": 73,
        "queue_type": "RANKED_TFT",
    }
]

MOCK_ACTIVE_GAME = {
    "in_game": False,
    "game_id": None,
    "game_start_time": None,
}

MOCK_REGION = {
    "puuid": MOCK_PUUID,
    "game": "tft",
    "region": "asia",
}

MOCK_CHALLENGER = {
    "tier": "CHALLENGER",
    "queue": "RANKED_TFT",
    "name": "HexForge Challengers",
    "entries": [
        {"puuid": "p1", "summoner_id": "s1", "tier": "CHALLENGER", "rank": "I", "league_points": 1423, "wins": 210, "losses": 145, "queue_type": "RANKED_TFT"},
        {"puuid": "p2", "summoner_id": "s2", "tier": "CHALLENGER", "rank": "I", "league_points": 1387, "wins": 198, "losses": 152, "queue_type": "RANKED_TFT"},
        {"puuid": "p3", "summoner_id": "s3", "tier": "CHALLENGER", "rank": "I", "league_points": 1356, "wins": 205, "losses": 148, "queue_type": "RANKED_TFT"},
        {"puuid": "p4", "summoner_id": "s4", "tier": "CHALLENGER", "rank": "I", "league_points": 1298, "wins": 187, "losses": 160, "queue_type": "RANKED_TFT"},
        {"puuid": "p5", "summoner_id": "s5", "tier": "CHALLENGER", "rank": "I", "league_points": 1219, "wins": 178, "losses": 169, "queue_type": "RANKED_TFT"},
    ],
}

MOCK_GRANDMASTER = {
    "tier": "GRANDMASTER",
    "queue": "RANKED_TFT",
    "name": "HexForge Grandmasters",
    "entries": [
        {"puuid": "p6", "summoner_id": "s6", "tier": "GRANDMASTER", "rank": "I", "league_points": 782, "wins": 165, "losses": 140, "queue_type": "RANKED_TFT"},
        {"puuid": "p7", "summoner_id": "s7", "tier": "GRANDMASTER", "rank": "I", "league_points": 743, "wins": 158, "losses": 147, "queue_type": "RANKED_TFT"},
        {"puuid": "p8", "summoner_id": "s8", "tier": "GRANDMASTER", "rank": "I", "league_points": 698, "wins": 152, "losses": 153, "queue_type": "RANKED_TFT"},
        {"puuid": "p9", "summoner_id": "s9", "tier": "GRANDMASTER", "rank": "I", "league_points": 654, "wins": 148, "losses": 157, "queue_type": "RANKED_TFT"},
        {"puuid": "p10", "summoner_id": "s10", "tier": "GRANDMASTER", "rank": "I", "league_points": 612, "wins": 143, "losses": 162, "queue_type": "RANKED_TFT"},
    ],
}

MOCK_MASTER = {
    "tier": "MASTER",
    "queue": "RANKED_TFT",
    "name": "HexForge Masters",
    "entries": [
        {"puuid": "p11", "summoner_id": "s11", "tier": "MASTER", "rank": "I", "league_points": 479, "wins": 138, "losses": 135, "queue_type": "RANKED_TFT"},
        {"puuid": "p12", "summoner_id": "s12", "tier": "MASTER", "rank": "I", "league_points": 423, "wins": 132, "losses": 141, "queue_type": "RANKED_TFT"},
        {"puuid": "p13", "summoner_id": "s13", "tier": "MASTER", "rank": "I", "league_points": 378, "wins": 128, "losses": 145, "queue_type": "RANKED_TFT"},
        {"puuid": "p14", "summoner_id": "s14", "tier": "MASTER", "rank": "I", "league_points": 341, "wins": 125, "losses": 148, "queue_type": "RANKED_TFT"},
        {"puuid": "p15", "summoner_id": "s15", "tier": "MASTER", "rank": "I", "league_points": 298, "wins": 121, "losses": 152, "queue_type": "RANKED_TFT"},
    ],
}

MOCK_PLATFORM_STATUS = {
    "id": "kr",
    "name": "Korea",
    "locales": ["ko_KR"],
    "maintenances": [],
    "incidents": [],
}


class MockAPIHandler(http.server.BaseHTTPRequestHandler):
    LANDING_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "landing")

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)

        # API health check
        if parsed.path == "/api/health":
            self._json(200, {"status": "ok", "mode": "mock"})

        # Download page or binary
        elif parsed.path.startswith("/download/"):
            self._serve_download(parsed.path)

        # Redirect /app to the Vite dev server (or Tauri app)
        elif parsed.path == "/app" or parsed.path == "/app/":
            self.send_response(302)
            self.send_header("Location", "http://raspberrypi.local:1420/")
            self.end_headers()

        # Landing page at root
        elif parsed.path == "/" or parsed.path == "/index.html":
            self._serve_landing("index.html")

        # Static assets from landing/ directory
        elif parsed.path.startswith("/landing/"):
            self._serve_static(parsed.path)

        # Favicon
        elif parsed.path.startswith("/favicon-"):
            self._serve_favicon(parsed.path)

        else:
            self._json(404, {"error": "not found"})

    FAVICON_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "src-tauri", "icons")

    def _serve_favicon(self, path):
        """Serve favicon files from src-tauri/icons/."""
        filename = os.path.basename(path)
        filepath = os.path.join(self.FAVICON_DIR, filename)
        if not os.path.isfile(filepath):
            # Fallback to any available PNG
            for f in ["32x32.png", "128x128.png"]:
                fp = os.path.join(self.FAVICON_DIR, f)
                if os.path.isfile(fp):
                    filepath = fp
                    break
        if os.path.isfile(filepath):
            with open(filepath, "rb") as f:
                data = f.read()
            mime = "image/png" if filepath.endswith(".png") else "image/x-icon"
            self.send_response(200)
            self.send_header("Content-Type", mime)
            self.send_header("Cache-Control", "public, max-age=3600")
            self.end_headers()
            self.wfile.write(data)
        else:
            self._json(404, {"error": "favicon not found"})

    def _serve_landing(self, filename):
        """Serve a file from the landing/ directory."""
        filepath = os.path.join(self.LANDING_DIR, filename)
        if not os.path.isfile(filepath):
            self._json(404, {"error": "not found"})
            return
        with open(filepath, "rb") as f:
            data = f.read()
        ext = os.path.splitext(filename)[1].lower()
        mime = {
            ".html": "text/html; charset=utf-8",
            ".css": "text/css; charset=utf-8",
            ".js": "application/javascript; charset=utf-8",
            ".png": "image/png",
            ".jpg": "image/jpeg",
            ".jpeg": "image/jpeg",
            ".svg": "image/svg+xml",
            ".ico": "image/x-icon",
        }.get(ext, "application/octet-stream")
        self.send_response(200)
        self.send_header("Content-Type", mime)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(data)
        print(f"[proxy] Served landing/{filename} ({len(data)} bytes)")

    def _serve_static(self, path):
        """Serve a static file from landing/assets/."""
        # Remove leading / and sanitize
        rel_path = path.lstrip("/")
        filepath = os.path.join(os.path.dirname(os.path.abspath(__file__)), rel_path)
        if not os.path.isfile(filepath):
            self._json(404, {"error": "not found"})
            return
        with open(filepath, "rb") as f:
            data = f.read()
        ext = os.path.splitext(filepath)[1].lower()
        mime = {
            ".html": "text/html; charset=utf-8",
            ".css": "text/css; charset=utf-8",
            ".js": "application/javascript; charset=utf-8",
            ".png": "image/png",
            ".jpg": "image/jpeg",
            ".jpeg": "image/jpeg",
            ".svg": "image/svg+xml",
            ".ico": "image/x-icon",
        }.get(ext, "application/octet-stream")
        self.send_response(200)
        self.send_header("Content-Type", mime)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(data)
        print(f"[proxy] Served {path} ({len(data)} bytes)")

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

        elif parsed.path == "/api/get-player-rank":
            self._json(200, MOCK_RANK)

        elif parsed.path == "/api/get-active-game-status":
            self._json(200, MOCK_ACTIVE_GAME)

        elif parsed.path == "/api/refresh-matches":
            count = data.get("count", 10)
            self._json(200, {"fetched": count, "new_matches": min(3, count), "errors": 0})

        elif parsed.path == "/api/get-player-region":
            self._json(200, MOCK_REGION)

        elif parsed.path == "/api/get-challenger-standings":
            self._json(200, MOCK_CHALLENGER)

        elif parsed.path == "/api/get-grandmaster-standings":
            self._json(200, MOCK_GRANDMASTER)

        elif parsed.path == "/api/get-master-standings":
            self._json(200, MOCK_MASTER)

        elif parsed.path == "/api/get-platform-status":
            self._json(200, MOCK_PLATFORM_STATUS)

        # Favicon
        elif parsed.path.startswith("/favicon-"):
            self._serve_favicon(parsed.path)

        else:
            self._json(404, {"error": "not found"})

    def _serve_download(self, path):
        """Serve download page or binary file."""
        # If requesting the .exe itself
        if path.endswith(".exe"):
            exe_path = os.path.join(DOWNLOAD_DIR, "hexforge-companion.exe")
            if os.path.exists(exe_path):
                file_size = os.path.getsize(exe_path)
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Disposition", 'attachment; filename="hexforge-companion.exe"')
                self.send_header("Content-Length", str(file_size))
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                with open(exe_path, "rb") as f:
                    self.wfile.write(f.read())
                return
            self._json(404, {"error": "Binary not found — build may still be in progress."})
            return

        # Serve download page
        exe_exists = os.path.exists(os.path.join(DOWNLOAD_DIR, "hexforge-companion.exe"))
        exe_size = 0
        if exe_exists:
            exe_size = os.path.getsize(os.path.join(DOWNLOAD_DIR, "hexforge-companion.exe"))

        page = f"""<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8">
<title>HexForge Companion — Download</title>
<style>
  body {{ background: #0d0d1a; color: #f0f0f0; font-family: 'Segoe UI', sans-serif; margin: 40px; text-align: center; }}
  .card {{ background: rgba(0,0,0,0.45); backdrop-filter: blur(4px); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; padding: 24px; max-width: 500px; margin: 40px auto; }}
  h1 {{ color: #c8a84e; font-size: 18px; }}
  .btn {{ display: inline-block; background: #c8a84e; color: #000; padding: 10px 24px; border-radius: 6px; text-decoration: none; font-weight: 700; margin: 12px 0; }}
  .btn:hover {{ background: #d4b85a; }}
  .btn:disabled {{ opacity: 0.4; cursor: not-allowed; }}
  .status {{ font-size: 12px; color: #888; margin: 8px 0; }}
  .info {{ font-size: 11px; color: #555; text-align: left; margin-top: 16px; }}
  .info li {{ margin: 4px 0; }}
  .footer {{ font-size: 9px; color: rgba(255,255,255,0.2); margin-top: 24px; }}
</style></head>
<body>
  <div class="card">
    <h1>⬇ HexForge Companion</h1>
    <p style="color:#aaa;font-size:13px;">Windows x86_64 — Release Build</p>
    {f'<a class="btn" href="/download/hexforge-companion.exe">Download .exe ({exe_size // 1024 // 1024} MB)</a>' if exe_exists else '<button class="btn" disabled>Build in progress...</button>'}
    <div class="status">{f'✅ Build complete — {exe_size // 1024 // 1024} MB' if exe_exists else '⏳ Cross-compilation is running...'}</div>
    <div class="info">
      <strong>Zero config:</strong> Download and run — the app works in <strong>Mock mode</strong> out of the box with no API key needed. Full features work immediately.
      <br><br>
      <strong>Optional: Live data with API key</strong>
      <ol>
        <li>Create a <code>.env</code> file next to the <code>.exe</code></li>
        <li>Add <code>RGAPI_KEY=RGAPI-xxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx</code></li>
        <li>Restart the app — automatically switches to live API mode</li>
      </ol>
      <p style="font-size:10px;color:#888;">No API key? Mock mode uses pre-built mock data for all features.</p>
    </div>
    <p style="font-size:10px;color:#555;">Built from <a href="https://github.com/fyosua/HexForge-Companion" style="color:#5dade2;">github.com/fyosua/HexForge-Companion</a></p>
  </div>
  <div class="footer">HexForge Companion isn't endorsed by Riot Games.</div>
</body></html>"""
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(page.encode())

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
    print(f"[HexForge Proxy] Endpoints:")
    print(f"  GET  /                        Landing page (Riot review)")
    print(f"  GET  /app                     Redirect to frontend app")
    print(f"  GET  /landing/assets/*        Static mockup assets")
    print(f"  GET  /api/health")
    print(f"  POST /api/resolve-player")
    print(f"  POST /api/get-match-history")
    print(f"  POST /api/get-player-stats")
    print(f"  POST /api/get-player-rank")
    print(f"  POST /api/get-active-game-status")
    print(f"  POST /api/refresh-matches")
    print(f"  POST /api/get-player-region")
    print(f"  POST /api/get-challenger-standings")
    print(f"  POST /api/get-grandmaster-standings")
    print(f"  POST /api/get-master-standings")
    print(f"  POST /api/get-platform-status")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[HexForge Proxy] Shutdown")
        server.server_close()