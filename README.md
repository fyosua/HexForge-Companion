<<<<<<< HEAD
# HexForge-Companion ⚒️🔮
A lightweight, high-performance, and Riot-compliant Teamfight Tactics (TFT) desktop companion app built with Tauri v2, Rust (Riven), and optimized SQLite. Focuses on post-game flexibility analytics, pre-game boards, and static transition planning without live-game decision dictation.

Unlike traditional, resource-heavy Electron or Overwolf-dependent apps that bundle an entire Chromium browser and consume hundreds of megabytes of RAM, HexForge is engineered from the ground up using Tauri v2 and Rust. It runs on a native OS Webview, keeping its footprint under 45 MB of RAM and maintaining near $O(1)$ CPU thread scheduling to ensure your in-game frame rates never stutter.  

HexForge operates as a Hybrid Strategy Engine, bridging the gap between raw win-rate numbers (which can trap players into rigid playstyles) and subjective expert tier lists. It empowers players to understand the why behind tactical shifts and transition boards, focusing on post-game reflection and pre-game preparation.  

🚀 Key Features
Non-Intrusive Transparent Overlay: A beautiful, frameless overlay that renders seamlessly over borderless windowed TFT matches.

Dynamic Click-Through HUD: Uses OS-level mouse state polling to instantly toggle cursor ignore states (set_ignore_cursor_events). This allows you to click "behind" the app's transparent elements to shop or move units without interference.

Post-Game "Flexibility Delta" Analysis: Evaluates your Stage 2-1 board state combinations against your final Stage 5-1 composition, giving you clear analytical data on transition execution and missed pivot windows.  

Static Pre-Game Planning: Highlight transition pipelines, item slams, and composition trees before the loading screen fades.  

Localized Asset Caching: All static assets (champions, traits, items) from TFT Data Dragon and Community Dragon are cached directly on your disk, maximizing load speed and eliminating redundant network traffic.

🛠️ Tech Stack & Architecture
HexForge's dual-process architecture is built for maximum resource efficiency and runtime safety:

Frontend UI: Light HTML5/CSS3/TypeScript rendered via native operating system engines (WebView2/Chromium on Windows, WebKit on macOS).

System Backend (Rust): High-speed, memory-safe backend handling native windowing, disk I/O, and secure network routing.

Riot API Client (Riven): A robust, thread-safe asynchronous Rust library that manages Riot Games API interactions and dynamically parses rate limits.  

Local Cache Engine (SQLite WAL): An embedded relational database configured with high-performance runtime pragmas (WAL journal mode, sequential batch transactions, and prepared statements) to prevent storage bottlenecks and protect SSD lifespans .
┌───────────────────────────────────┐
│     Tauri v2 Front-end Webview    │
│  (HTML5 / CSS / React / SolidJS)  │
└─────────────────┬─────────────────┘
│ Tauri IPC (Commands / Events)
▼
┌───────────────────────────────────┐
│          Rust Core Backend        │
│  (Tokio Async Runtime / Riven API)│
└─────────────────┬─────────────────┘
│ Local I/O
▼
┌───────────────────────────────────┐
│        SQLite Database (WAL)      │
│     (%LOCALAPPDATA%\HexForge)    │
└───────────────────────────────────┘
=======
<div align="center">

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
>>>>>>> c84fd0f (feat: HexForge Companion Track 1 — Dev environment, DB, overlay, mock pipeline)
