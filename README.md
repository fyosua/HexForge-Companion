# HexForge Companion ⚔️

**A lightweight, compliance-hardened TFT desktop companion overlay**

Built with **Rust + Tauri v2** — no Overwolf, no bloat, just a transparent frameless overlay that respects Riot Games' competitive integrity policies.

---

## ✨ Features

- 🪟 **Transparent overlay** — sits on top of your TFT game client, click-through when idle
- 🔍 **Player search** — resolve Riot IDs → PUUID → summoner info in two steps
- 📊 **Match history** — view recent games with placement, game version, and timestamps
- 📈 **Stats dashboard** — win rate, avg placement, top 4 rate (all compliance-safe)
- 🥇 **Rank display** — TFT ranked tier, LP, wins/losses per queue
- 🟢 **Live match status** — green/red dot indicator for active game (no opponent data)
- 🏆 **Leaderboard** — Challenger, Grandmaster, and Master standings per platform
- 🩺 **Platform status** — maintenance and incident alerts from Riot
- 📌 **Pinned widget** — compact mini-dashboard with rank + game status, stays onscreen while playing
- 🔒 **3 API modes** — Mock (offline), Direct (dev key), Proxy (production key behind backend)
- 🗄️ **Local SQLite** — WAL-mode database caches all match data locally
- 🛡️ **Riot-compliant** — no augment/legend win rates, no live-scouting, no decision dictation
- ⚖️ **GDPR-ready** — one-click account data deletion with cascade wipe
- 🖥️ **Browser preview** — run `python3 proxy.py` for mock API testing at `http://0.0.0.0:1420`
- 🪟 **Windows cross-compilation** — build from Linux with `x86_64-pc-windows-gnu` target

## 🔧 Tech Stack

| Layer | Technology |
|-------|-----------|
| **Backend** | Rust, Tauri v2, tokio async runtime |
| **Database** | SQLite 3 with WAL journal mode |
| **API Client** | reqwest with 3-mode abstraction |
| **Frontend** | React 18 + TypeScript + Vite |
| **Overlay** | WebView2 frameless transparent window |
| **Riot API** | Riven-compatible endpoints via proxy or direct |

## 📥 Download

Pre-built binaries are available from the **local download page** at:

```
http://raspberrypi.local:8000/download/
```

Or build from source (see below).

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 18+ (for frontend build)
- WebView2 Runtime (Windows) / WebKit2GTK (Linux)
- Riot Games API key from [developer.riotgames.com](https://developer.riotgames.com)

### Setup

```bash
# Clone & enter project
git clone https://github.com/fyosua/HexForge-Companion.git
cd HexForge-Companion

# Copy environment config
cp .env.example .env
# Edit .env with your RGAPI_KEY and region settings

# Install frontend dependencies
npm install

# Build
npm run build         # Frontend
cd src-tauri && cargo build  # Rust backend
```

### Running

```bash
# Development mode (hot-reload frontend + Tauri)
npm run tauri dev

# Run built binary
./src-tauri/target/debug/hexforge-companion
```

### Browser Preview (no Tauri needed)

```bash
# Terminal 1: Start mock API proxy
python3 proxy.py

# Terminal 2: Start Vite dev server
npm run dev

# Open in browser
# → http://raspberrypi.local:1420 (or 0.0.0.0:1420)
```

## 🎮 API Modes

HexForge Companion supports three operational modes, auto-detected from `.env`:

| Mode | Env Config | When to Use |
|------|-----------|-------------|
| **Mock** | `USE_MOCK=true` (or no key set) | Offline development, no API key needed |
| **Direct** | `RGAPI_KEY` is set | Personal / Development key from Riot portal |
| **Proxy** | `RIOT_PROXY_URL` is set | Production deployment with secure key backend |

## 🗺️ Project Architecture

```
hexforge-companion/
├── src/                    # React frontend
│   ├── components/
│   │   ├── PlayerSearch.tsx
│   │   ├── MatchHistory.tsx
│   │   ├── PlayerStats.tsx
│   │   ├── RankDisplay.tsx
│   │   ├── InGameIndicator.tsx
│   │   ├── LeaderboardDisplay.tsx
│   │   ├── PlatformStatus.tsx
│   │   ├── PinnedWidget.tsx
│   │   ├── DisplayModeWarning.tsx
│   │   └── LegalFooter.tsx
│   ├── App.tsx
│   └── App.css
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # App state, env loading, Tauri setup
│   │   ├── api.rs          # Riot API client (3 modes, all DTOs)
│   │   ├── db.rs           # SQLite WAL database layer
│   │   ├── commands.rs     # 14 Tauri IPC command handlers
│   │   └── overlay.rs      # Cursor passthrough logic
│   ├── mock/               # 15 offline mock JSON responses
│   └── tauri.conf.json     # Transparent overlay window config
├── proxy.py                # Mock API proxy for browser preview (port 1421)
├── .env                    # Local secrets (gitignored)
├── .env.example            # Example config
├── .cargo/config.toml      # Cross-compilation linker config
└── docs/                   # Detailed documentation
```

## 🪟 Windows Cross-Compilation

Build Windows binaries from Linux using MinGW:

```bash
# Install MinGW
sudo apt install gcc-mingw-w64-x86-64

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build
cd src-tauri
cargo build --target x86_64-pc-windows-gnu

# Binary at:
# src-tauri/target/x86_64-pc-windows-gnu/debug/hexforge-companion.exe
```

Linker is configured in `.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

## 🛡️ Riot Games Compliance

This application is designed in full compliance with Riot Games' Third-Party Application Policies:

- ✅ **No augment/legend win rates** — stats display only aggregate placement data
- ✅ **No live-scouting** — `get_active_game_status` returns only in_game bool + start time; no opponent data
- ✅ **No decision dictation** — the app shows data, not instructions
- ✅ **Riot legal boilerplate** — displayed on every screen via `LegalFooter`
- ✅ **GDPR account deletion** — one-click data wipe via `request_account_deletion`
- ✅ **Leaderboard display** — public Challenger/Grandmaster/Master standings only

See [docs/COMPLIANCE.md](docs/COMPLIANCE.md) for the full compliance audit.

## 🔄 Development Roadmap

- **Track 1 (Dev)** ✅ Environment, DB schema, overlay window, mock pipeline
- **Track 2 (Personal)** ✅ Live API integration, match ingestion, leaderboard, cross-compilation
- **Track 3 (Production)** ⬜ RSO OAuth, production key, macOS build

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

## ⚠️ Disclaimer

HexForge Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.