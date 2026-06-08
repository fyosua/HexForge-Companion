# HexForge Companion — Architecture

## Overview

HexForge Companion is a **Tauri v2** desktop application that provides a lightweight, transparent overlay for Teamfight Tactics. The Rust backend handles all data ingestion, local caching, and Riot API communication while the React frontend renders the HUD interface.

## Core Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri v2 Shell                         │
│  ┌─────────────────────┐    ┌────────────────────────────┐  │
│  │   Rust Backend       │    │   React Frontend           │  │
│  │                      │    │                            │  │
│  │  ┌───────────────┐   │    │  ┌──────────────────────┐ │  │
│  │  │  RiotApiClient │──┼────┼──│  PlayerSearch.tsx     │ │  │
│  │  │  (Mock/Direct/ │   │    │  └──────────────────────┘ │  │
│  │  │   Proxy)       │   │    │  ┌──────────────────────┐ │  │
│  │  └───────┬───────┘   │    │  │  MatchHistory.tsx     │ │  │
│  │          │           │    │  └──────────────────────┘ │  │
│  │  ┌───────▼───────┐   │    │  ┌──────────────────────┐ │  │
│  │  │  Database      │   │    │  │  PlayerStats.tsx     │ │  │
│  │  │  (SQLite WAL)  │   │    │  └──────────────────────┘ │  │
│  │  └───────────────┘   │    │  ┌──────────────────────┐ │  │
│  │                      │    │  │  LegalFooter.tsx      │ │  │
│  │  ┌───────────────┐   │    │  └──────────────────────┘ │  │
│  │  │  Overlay       │   │    │  ┌──────────────────────┐ │  │
│  │  │  (cursor       │   │    │  │  DisplayModeWarning  │ │  │
│  │  │   passthrough) │   │    │  └──────────────────────┘ │  │
│  │  └───────────────┘   │    │                            │  │
│  └─────────────────────┘    └────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Riot API (v1)                            │
│  /riot/account/v1/accounts/by-riot-id/{name}/{tag}          │
│  /tft/summoner/v1/summoners/by-puuid/{platform}/{puuid}    │
│  /tft/match/v1/matches/by-puuid/{puuid}/ids                │
│  /tft/match/v1/matches/{match_id}                           │
└─────────────────────────────────────────────────────────────┘
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

When the mouse is not over a HUD element (`<main class="hex-hud-interactive">`), cursor events pass through to the game via `set_ignore_cursor_events(true)`.

## API Client Modes

### Mock Mode
- Reads from `src-tauri/mock/*.json` files
- Requires `USE_MOCK=true` in `.env` (or no API key set)
- Returns realistic sample match data

### Direct Mode  
- Passes `X-Riot-Token` header directly to Riot API
- Requires `RGAPI_KEY` in `.env`
- Uses `RIOT_REGION` for routing (`americas`, `asia`, `europe`)
- Good for development and personal keys

### Proxy Mode
- Routes all requests through a backend proxy
- Requires `RIOT_PROXY_URL` in `.env`
- API key lives on the backend — never in the binary
- Required for production deployment with RSO

## Database Schema

```sql
-- WAL mode optimization
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA cache_size = -8192;
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;

CREATE TABLE IF NOT EXISTS players (
    puuid TEXT PRIMARY KEY,
    game_name TEXT NOT NULL,
    tag_line TEXT NOT NULL,
    summoner_id TEXT,
    summoner_level INTEGER DEFAULT 0,
    profile_icon_id INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS matches (
    match_id TEXT PRIMARY KEY,
    puuid TEXT NOT NULL REFERENCES players(puuid) ON DELETE CASCADE,
    game_datetime INTEGER NOT NULL,
    game_length REAL,
    placement INTEGER,
    game_version TEXT,
    tft_set_canonical TEXT,
    queue_id INTEGER,
    companion TEXT,
    traits TEXT,
    units TEXT,
    augments TEXT,
    total_damage_to_players INTEGER,
    last_round INTEGER,
    level INTEGER,
    player_level INTEGER,
    created_at TEXT DEFAULT (datetime('now'))
);
```

## IPC Commands

| Command | Direction | Description |
|---------|-----------|-------------|
| `resolve_player` | Frontend → Backend | Riot ID → PUUID + summoner info |
| `get_match_history` | Frontend → Backend | Recent matches from local DB |
| `get_player_stats` | Frontend → Backend | Aggregate placement stats |
| `hud_bounds_enter` | Frontend → Backend | Enable cursor on Hover |
| `hud_bounds_leave` | Frontend → Backend | Pass cursor through |
| `request_account_deletion` | Frontend → Backend | GDPR data purge |