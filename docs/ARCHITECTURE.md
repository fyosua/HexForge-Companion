# HexForge Companion — Architecture

## Overview

HexForge Companion is a **Tauri v2** desktop application that provides a lightweight, transparent overlay for Teamfight Tactics. The Rust backend handles all data ingestion, local caching, and Riot API communication while the React frontend renders the HUD interface. A Python mock proxy enables browser-based development without Tauri.

## Core Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                          Tauri v2 Shell                              │
│  ┌──────────────────────────┐    ┌────────────────────────────────┐  │
│  │     Rust Backend          │    │     React Frontend             │  │
│  │                           │    │                                │  │
│  │  ┌─────────────────────┐  │    │  ┌──────────────────────────┐ │  │
│  │  │    RiotApiClient     │──┼────┼──│  PlayerSearch.tsx         │ │  │
│  │  │  (Mock / Direct /    │  │    │  └──────────────────────────┘ │  │
│  │  │   Proxy)             │  │    │  ┌──────────────────────────┐ │  │
│  │  └───────┬─────────────┘  │    │  │  MatchHistory.tsx          │ │  │
│  │          │                │    │  └──────────────────────────┘ │  │
│  │  ┌───────▼─────────────┐  │    │  ┌──────────────────────────┐ │  │
│  │  │  SQLite Database     │  │    │  │  PlayerStats.tsx          │ │  │
│  │  │  (WAL mode,          │  │    │  └──────────────────────────┘ │  │
│  │  │   ~/.local/share/    │  │    │  ┌──────────────────────────┐ │  │
│  │  │   HexForge/db/)      │  │    │  │  RankDisplay.tsx          │ │  │
│  │  └──────────────────────┘  │    │  └──────────────────────────┘ │  │
│  │                           │    │  ┌──────────────────────────┐ │  │
│  │  ┌─────────────────────┐  │    │  │  InGameIndicator.tsx      │ │  │
│  │  │  Overlay             │  │    │  └──────────────────────────┘ │  │
│  │  │  (cursor passthrough │  │    │  ┌──────────────────────────┐ │  │
│  │  │   via                │  │    │  │  LeaderboardDisplay.tsx   │ │  │
│  │  │   set_ignore_cursor  │  │    │  └──────────────────────────┘ │  │
│  │  │   _events)           │  │    │  ┌──────────────────────────┐ │  │
│  │  └──────────────────────┘  │    │  │  PlatformStatus.tsx       │ │  │
│  │                           │    │  └──────────────────────────┘ │  │
│  └──────────────────────────┘    │  ┌──────────────────────────┐ │  │
│                                  │  │  PinnedWidget.tsx         │ │  │
│                                  │  └──────────────────────────┘ │  │
│                                  │  ┌──────────────────────────┐ │  │
│                                  │  │  DisplayModeWarning.tsx   │ │  │
│                                  │  └──────────────────────────┘ │  │
│                                  │  ┌──────────────────────────┐ │  │
│                                  │  │  LegalFooter.tsx          │ │  │
│                                  │  └──────────────────────────┘ │  │
│                                  └────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                          Riot Games API (v1/v5)                               │
│                                                                               │
│  ACCOUNT-V1:  /riot/account/v1/accounts/by-riot-id/{name}/{tag}              │
│  ACCOUNT-V1:  /riot/account/v1/active-shards/by-game/{game}/by-puuid/{puuid} │
│  SUMMONER-V1: /tft/summoner/v1/summoners/by-puuid/{platform}/{puuid}         │
│  MATCH-V1:    /tft/match/v1/matches/by-puuid/{puuid}/ids                     │
│  MATCH-V1:    /tft/match/v1/matches/{match_id}                                │
│  LEAGUE-V1:   /tft/league/v1/entries/by-puuid/{puuid}                        │
│  LEAGUE-V1:   /tft/league/v1/challenger                                       │
│  LEAGUE-V1:   /tft/league/v1/grandmaster                                      │
│  LEAGUE-V1:   /tft/league/v1/master                                           │
│  SPECTATOR-V5:/tft/spectator/v5/active-games/by-puuid/{puuid}                │
│  STATUS-V1:   /tft/status/v1/platform-data                                    │
└───────────────────────────────────────────────────────────────────────────────┘
```

## Window Configuration (tauri.conf.json)

The overlay window is configured as:

| Property | Value | Purpose |
|----------|-------|---------|
| `decorations` | `false` | No title bar — pure overlay |
| `transparent` | `true` | Background sees through to game |
| `alwaysOnTop` | `true` | Stays above TFT client |
| `skipTaskbar` | `true` | Doesn't appear as separate task |
| `focus` | `false` | Never steals focus from game |
| `resizable` | `true` | Can be resized for different resolutions |

When the mouse is not over a HUD element (`<main class="hex-hud-interactive">`), cursor events pass through to the game via `set_ignore_cursor_events(true)`. The `overlay.rs` module handles this with two functions:
- `set_passthrough(window, enabled)` — wraps `window.set_ignore_cursor_events()`
- `init_overlay(window)` — positions overlay on primary monitor at full resolution

## Frontend Components (10)

| Component | File | Purpose |
|-----------|------|---------|
| `PlayerSearch` | `PlayerSearch.tsx` | Riot ID input → resolve_player → sets active PUUID |
| `MatchHistory` | `MatchHistory.tsx` | Table of recent matches from local DB |
| `PlayerStats` | `PlayerStats.tsx` | Aggregate placement stats (win rate, avg placement, top4) |
| `RankDisplay` | `RankDisplay.tsx` | TFT rank, tier, LP, wins/losses per queue |
| `InGameIndicator` | `InGameIndicator.tsx` | Green/red dot showing whether player is in a live game |
| `LeaderboardDisplay` | `LeaderboardDisplay.tsx` | Challenger/Grandmaster/Master standings table |
| `PlatformStatus` | `PlatformStatus.tsx` | Maintenance and incident alerts per platform |
| `PinnedWidget` | `PinnedWidget.tsx` | Compact mini-dashboard pinned during gameplay |
| `DisplayModeWarning` | `DisplayModeWarning.tsx` | Fullscreen warning for overlay mode |
| `LegalFooter` | `LegalFooter.tsx` | Riot Games disclaimer on every screen |

## API Client Modes

### Mock Mode
- Reads from `src-tauri/mock/*.json` files (15 files covering account, summoner, match, league, spectator, status)
- Requires `USE_MOCK=true` in `.env` (or no API key set)
- Returns realistic sample match data
- Also accessible via `proxy.py` for browser preview

### Direct Mode  
- Passes `X-Riot-Token` header directly to Riot API
- Requires `RGAPI_KEY` in `.env`
- Uses `RIOT_REGION` (americas/asia/europe/sea) and `RIOT_PLATFORM` (kr/na1/etc.)
- Good for development and personal keys

### Proxy Mode
- Routes all requests through a backend proxy
- Requires `RIOT_PROXY_URL` in `.env`
- API key lives on the backend — never in the binary
- Required for production deployment with RSO
- Proxy endpoints use format: `{proxy_base}/api/riot/v1/...`

## Mock API Proxy (proxy.py)

A standalone Python HTTP server on port 1421 that serves mock responses for all IPC commands. Used for browser preview when Tauri is not available.

**Usage:**
```bash
python3 proxy.py
# → [HexForge Proxy] Mock API running on http://0.0.0.0:1421
```

The frontend auto-detects Tauri presence. When running in browser (`!isTauri()`), all `invoke()` calls are proxied through `http://raspberrypi.local:1421/api/{command}`.

## Database Schema

```
Location: ~/.local/share/HexForge/db/storage.db
Journal:  WAL (Write-Ahead Logging)
```

### Tables

**players** — one row per resolved Riot ID:
```
puuid TEXT PRIMARY KEY
game_name TEXT NOT NULL
tag_line TEXT NOT NULL
summoner_id TEXT
summoner_level INTEGER DEFAULT 0
profile_icon_id INTEGER DEFAULT 0
created_at TEXT DEFAULT datetime('now')
updated_at TEXT DEFAULT datetime('now')
```

**matches** — one row per participant per match:
```
match_id TEXT PRIMARY KEY
puuid TEXT NOT NULL → players(puuid) ON DELETE CASCADE
game_datetime INTEGER NOT NULL
game_length REAL
placement INTEGER
game_version TEXT
tft_set_canonical TEXT
queue_id INTEGER
companion TEXT
traits TEXT
units TEXT
augments TEXT
total_damage_to_players INTEGER
last_round INTEGER
level INTEGER
player_level INTEGER
created_at TEXT DEFAULT datetime('now')
```

**WAL pragmas:**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA cache_size = -8192;
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;
```

## IPC Commands (14 Total)

| Command | Direction | Params | Description |
|---------|-----------|--------|-------------|
| `resolve_player` | FE → BE | gameName, tagLine, platform | Riot ID → PUUID + summoner info |
| `get_match_history` | FE → BE | limit | Recent matches from local DB |
| `get_player_stats` | FE → BE | none | Aggregate placement stats |
| `get_player_rank` | FE → BE | none | TFT ranked league entries |
| `get_player_region` | FE → BE | none | Active region for linked player |
| `get_challenger_standings` | FE → BE | none | Challenger league standings |
| `get_grandmaster_standings` | FE → BE | none | Grandmaster league standings |
| `get_master_standings` | FE → BE | none | Master league standings |
| `get_platform_status` | FE → BE | none | Platform maintenance/incidents |
| `get_active_game_status` | FE → BE | none | In-game check (compliance-safe) |
| `refresh_matches` | FE → BE | count | Fetch + cache new matches from API |
| `hud_bounds_enter` | FE → BE | none | Enable cursor on Hover |
| `hud_bounds_leave` | FE → BE | none | Pass cursor through |
| `request_account_deletion` | FE → BE | none | GDPR data purge |

## Cross-Compilation

Windows x86_64 binaries can be built from Linux using MinGW cross-compilation.

**Toolchain:**
- `x86_64-w64-mingw32-gcc` (from `gcc-mingw-w64-x86-64`)
- Rust target: `x86_64-pc-windows-gnu`
- Linker configured in `.cargo/config.toml`

**Build:**
```bash
cargo build --target x86_64-pc-windows-gnu
```

**Output:** `src-tauri/target/x86_64-pc-windows-gnu/debug/hexforge-companion.exe`

## Startup Sequence

1. **lib.rs::run()** — loads `.env` from project root or app data dir
2. **ApiMode::from_env()** — auto-detects Mock/Direct/Proxy
3. **db::init_database()** — creates WAL database with schema
4. **Tauri builder** — manages AppState, registers all 14 IPC commands
5. **overlay::init_overlay()** — positions transparent window, enables cursor passthrough
6. **React mount** — components render, `useEffect` detects Tauri vs browser, attaches mouse event listeners