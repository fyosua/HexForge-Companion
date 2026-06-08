# HexForge Companion ⚔️

**A lightweight, compliance-hardened TFT desktop companion overlay**

Built with **Rust + Tauri v2** — no Overwolf, no bloat, just a transparent frameless overlay that respects Riot Games' competitive integrity policies.

</div>

---

## ✨ Features

- 🪟 **Transparent overlay** — sits on top of your TFT game client, click-through when idle
- 🔍 **Player search** — resolve Riot IDs → PUUID → summoner info in two steps
- 📊 **Match history** — view recent games with placement, game version, and timestamps
- 📈 **Stats dashboard** — win rate, avg placement, top 4 rate (all compliance-safe)
- 🔒 **3 API modes** — Mock (offline), Direct (dev key), Proxy (production key behind backend)
- 🗄️ **Local SQLite** — WAL-mode database caches all match data locally
- 🛡️ **Riot-compliant** — no augment/legend win rates, no live-scouting, no decision dictation
- ⚖️ **GDPR-ready** — one-click account data deletion with cascade wipe

## 🔧 Tech Stack

| Layer | Technology |
|-------|-----------|
| **Backend** | Rust, Tauri v2, tokio async runtime |
| **Database** | SQLite 3 with WAL journal mode |
| **API Client** | reqwest with 3-mode abstraction |
| **Frontend** | React 18 + TypeScript + Vite |
| **Overlay** | WebView2 frameless transparent window |
| **Riot API** | Riven-compatible endpoints via proxy or direct |

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
│   │   ├── LegalFooter.tsx
│   │   └── DisplayModeWarning.tsx
│   ├── App.tsx
│   └── App.css
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # App state, env loading, Tauri setup
│   │   ├── api.rs          # Riot API client (3 modes)
│   │   ├── db.rs           # SQLite WAL database layer
│   │   ├── commands.rs     # Tauri IPC command handlers
│   │   └── overlay.rs      # Cursor passthrough logic
│   ├── mock/               # Offline mock JSON responses
│   └── tauri.conf.json     # Transparent overlay window config
├── .env                    # Local secrets (gitignored)
├── .env.example            # Example config
└── docs/                   # Detailed documentation
```

## 🛡️ Riot Games Compliance

This application is designed in full compliance with Riot Games' Third-Party Application Policies:

- ✅ **No augment/legend win rates** — stats display only aggregate placement data
- ✅ **No live-scouting** — all analysis is post-match or pre-game static metadata
- ✅ **No decision dictation** — the app shows data, not instructions
- ✅ **Riot legal boilerplate** — displayed on every dashboard
- ✅ **GDPR account deletion** — one-click data wipe

See [docs/COMPLIANCE.md](docs/COMPLIANCE.md) for the full compliance audit.

## 🔄 Development Roadmap

- **Track 1 (Dev)** ✅ Environment, DB schema, overlay window, mock pipeline
- **Track 2 (Personal)** ⏳ Live API integration, rate limiter, match ingestion
- **Track 3 (Production)** ⬜ RSO OAuth, production key, macOS build

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

## ⚠️ Disclaimer

HexForge Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
