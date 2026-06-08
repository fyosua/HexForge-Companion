# HexForge Companion ⚔️

**A lightweight, compliance-hardened TFT desktop companion overlay**

Version **0.5.0** • Built with **Rust + Tauri v2** — no Overwolf, no bloat, transparent frameless overlay that auto-attaches to the TFT game window.

---

## ✨ Features

- 🪟 **Transparent overlay** — sits on top of TFT, stays out of your way
- 🎮 **Auto-attach** — detects TFT process launch, resizes overlay to match game window
- 🔔 **System tray** — show/hide/quit from the notification area; double-click to restore
- 🔒 **Pass-through mode** — mouse events pass through the overlay to the game, except interactive elements
- 📊 **Live rank & match data** — pull from Riot API or mock locally
- 🧩 **Pinned widget** — compact rank + game-status indicator, draggable, always on top
- 🔄 **Auto-refresh** — every 30s when pinned, manual refresh for full data
- 🛡️ **Riot-compliant** — no augment/legend win rates, no live scouting
- 💾 **Local caching** — SQLite with WAL mode, 0.5s cold start

## 💻 System Requirements

| Platform | Minimum | Recommended |
|----------|---------|-------------|
| **Windows** | Windows 10 64-bit, 4GB RAM | Windows 11, 8GB RAM |
| **macOS** | macOS 12.0+, 4GB RAM | macOS 14+, Apple Silicon |
| **Linux** | GTK3, WebKit2GTK 4.1, 4GB RAM | Wayland, 8GB RAM |
| **Storage** | 100MB free | 500MB free (for match history cache) |
| **TFT** | Installed via Riot Client | Latest patch |

**Runtime dependency:** [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (Windows) or system WebKit (macOS/Linux) — installed automatically on most modern systems.

## 🚀 Quick Start

### Windows (recommended)

**Zero config:** Download the .exe and run. The app works in **Mock mode** out of the box — no API key needed.

1. Grab the latest build:  
   `http://raspberrypi.local:1421/download/` (local dev) or build from source
2. Run `HexForge-Companion.exe`
3. The overlay appears. Search any player name to explore.

**Optional — live data:**
1. Get a [Riot Dev API Key](https://developer.riotgames.com/)
2. Create `.env` next to the `.exe`:  
   `RGAPI_KEY=RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
3. Restart the app — it auto-detects the key and switches to Direct mode.

### From Source (Raspberry Pi / Linux)

```bash
# Prerequisites: Rust, Node 18+, libwebkit2gtk-4.1
git clone https://github.com/fyosua/HexForge-Companion.git
cd HexForge-Companion

# Optional: configure API key
cp .env.example .env
# Edit .env to add your RGAPI_KEY

# Run in development mode
npm install
npm run tauri dev
```

### Cross-Compile for Windows (from Linux)

```bash
# Prerequisites: mingw-w64, cargo-xwin or zig toolchain
cd src-tauri
cargo build --target x86_64-pc-windows-gnu --release
# Binary at: src-tauri/target/x86_64-pc-windows-gnu/release/hexforge-companion.exe
```

## 🔧 Architecture

```
┌──────────────────────────────┐
│   React Frontend (.tsx)      │  WebView2 overlay
│   useTftWatcher (auto-resize)│
│   PinnedWidget (compact)     │
├──────────────────────────────┤
│   Tauri IPC Bridge           │  invoke() commands
├──────────────────────────────┤
│   Rust Backend               │  process watcher, API client, DB
│   ┌─ Process Watcher ──────┐ │  Win32/Linux TFT process detection
│   │  Polls every 2s        │ │  → emits tft-attached / tft-detached
│   │  Win32 FindWindowW     │ │
│   └────────────────────────┘ │
│   ┌─ Riot API Client ──────┐ │  3 modes: Mock / Direct / Proxy
│   │  Rate-limited, cached  │ │
│   └────────────────────────┘ │
│   ┌─ SQLite (WAL) ─────────┐ │  Player data, match history
│   └────────────────────────┘ │
└──────────────────────────────┘
```

## 🖥️ Overlay Lifecycle

1. App starts → watcher thread begins polling for TFT window (every 2s)
2. TFT launches → Win32 `FindWindowW("RiotWindowClass")` detects window
3. Rust backend emits `tft-attached` with window geometry `{x, y, width, height}`
4. Frontend `useTftWatcher` hook receives event, calls `appWindow.setPosition()` + `setSize()`
5. Overlay snaps to game window, appears on top
6. User interacts with overlay via pass-through interactive elements
7. TFT closes → watcher emits `tft-detached`
8. Frontend hides overlay, resets state
9. App stays in system tray — double-click tray icon to restore

## 🛡️ Compliance

HexForge Companion complies with all Riot Games Third-Party Application Policies:

- ✅ **No augment/legend win-rate display** — blocked at the API client level
- ✅ **No live scouting** — spectator data is not used or stored
- ✅ **No direct decision dictation** — displays data, does not tell you what to play
- ✅ **Legal boilerplate** — displayed on every dashboard

See [docs/COMPLIANCE.md](docs/COMPLIANCE.md) for the full audit.

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture, component tree, data flow |
| [COMPLIANCE.md](docs/COMPLIANCE.md) | Riot policy audit, Production API key requirements |
| [API.md](docs/API.md) | All IPC commands, Riot API endpoints, proxy routes |

## 🏗️ Project Structure

```
src-tauri/          # Rust backend
├── src/
│   ├── main.rs          # Entry point (windows_subsystem = "windows")
│   ├── lib.rs           # App setup, tray, watcher, state
│   ├── api.rs           # Riot API client (3 modes)
│   ├── commands.rs      # 14 Tauri IPC commands
│   ├── db.rs            # SQLite init + cache
│   ├── overlay.rs       # Overlay positioning helpers
│   └── process_watcher.rs # TFT window detection + events
├── icons/               # App + tray icons
└── tauri.conf.json      # Production configuration

src/                # React frontend
├── App.tsx             # Main dashboard layout
├── main.tsx            # React entry
├── hooks/
│   ├── useApi.ts       # Shared fetch hook (AbortController)
│   └── useTftWatcher.tsx # TFT attach/detach + auto-resize
├── components/
│   ├── PlayerSearch.tsx
│   ├── MatchHistory.tsx
│   ├── PlayerStats.tsx
│   ├── RankDisplay.tsx
│   ├── InGameIndicator.tsx
│   ├── LeaderboardDisplay.tsx
│   ├── PlatformStatus.tsx
│   ├── PinnedWidget.tsx
│   ├── LegalFooter.tsx
│   └── DisplayModeWarning.tsx
└── App.css             # Overlay styling

proxy.py            # Mock backend for browser preview
mock/               # Sample API response JSON files
```

## 📦 Building for Distribution

```bash
# Frontend only (static preview)
npm run build

# Full Tauri app (debug)
npm run tauri dev

# Production bundle
cd src-tauri
cargo build --release
# Wraps into NSIS .exe, .msi, .deb, .AppImage depending on platform
```

## 📄 License

MIT — see [LICENSE](LICENSE).

---

> HexForge Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
