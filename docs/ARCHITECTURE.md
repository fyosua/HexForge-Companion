# HexForge Companion — Architecture

## Overview

HexForge Companion is a **Tauri v2** desktop application that provides a lightweight, transparent overlay for Teamfight Tactics. The Rust backend handles all data ingestion, local caching, Riot API communication, and TFT process detection while the React frontend renders the overlay UI.

Version **0.5.0** — pre-release / early access.

---

## Core Principles

1. **Memory-safe Rust** — no unsafe code (except Win32 FFI) in the backend
2. **Riot compliance** — blocked augment/legend data, no live scouting, legal boilerplate on every view
3. **Overlay-first UX** — transparent window, pass-through mouse, auto-attach to TFT process
4. **Zero config** — mock mode out of the box, optional API key for live data
5. **Offline-capable** — SQLite cache with WAL mode, full feature set without API key

---

## Layer Diagram

```
┌──────────────────────────────────────────────────────────┐
│  WINDOW: Tauri WebView2 (transparent, frameless, AOT)    │
│  ┌────────────────────────────────────────────────────┐  │
│  │  BREACT FRONTEND                                    │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐           │  │
│  │  │App.tsx   │ │Dashboard │ │Pinned    │            │  │
│  │  │(router)  │ │components│ │Widget    │            │  │
│  │  └────┬─────┘ └──────────┘ └──────────┘           │  │
│  │       │ useTftWatcher hook                         │  │
│  │       │   → auto-resize on tft-attached           │  │
│  │       │   → auto-hide on tft-detached             │  │
│  │       │ useApi hook                                │  │
│  │       │   → invoke() or proxy fetch               │  │
│  └───────┼────────────────────────────────────────────┘  │
└──────────┼───────────────────────────────────────────────┘
           │ Tauri IPC (invoke / events)
┌──────────┼───────────────────────────────────────────────┐
│  RUST BACKEND                                            │
│  ┌───────┴──────────────────────────────────────────┐    │
│  │  lib.rs (AppState + setup)                       │    │
│  │  ├── TrayIconBuilder (show/hide/quit)            │    │
│  │  ├── process_watcher::spawn_watcher()            │    │
│  │  └── commands.rs (14 invoke handlers)            │    │
│  ├──────────────────────────────────────────────────┤    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌───────────┐│    │
│  │  │ Process     │  │ Riot API    │  │ SQLite    ││    │
│  │  │ Watcher     │  │ Client      │  │ (WAL)     ││    │
│  │  │ (2s poll)   │  │ 3 modes     │  │ cache     ││    │
│  │  └─────────────┘  └─────────────┘  └───────────┘│    │
│  └──────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘
```

---

## Component Tree

### Rust Backend

| Module | File | Responsibility |
|--------|------|---------------|
| `lib.rs` | `src-tauri/src/lib.rs` | App entry, state, tray icon, watcher lifecycle |
| `main.rs` | `src-tauri/src/main.rs` | `windows_subsystem = "windows"`, calls `lib::run()` |
| `api.rs` | `src-tauri/src/api.rs` | Riot API client (3 modes), rate limiting, DTOs |
| `commands.rs` | `src-tauri/src/commands.rs` | 14 Tauri IPC `#[command]` handlers |
| `db.rs` | `src-tauri/src/db.rs` | SQLite init (WAL pragma), schema, migrations |
| `overlay.rs` | `src-tauri/src/overlay.rs` | hit-test / bounds helpers for pass-through |
| `process_watcher.rs` | `src-tauri/src/process_watcher.rs` | TFT window detection, event emission |

### React Frontend

| Component | File | Purpose |
|-----------|------|---------|
| App | `src/App.tsx` | Dashboard layout, state management, refresh logic |
| PlayerSearch | `src/components/PlayerSearch.tsx` | Summoner search input + Riot ID resolution |
| MatchHistory | `src/components/MatchHistory.tsx` | Recent match list with placements/comps |
| PlayerStats | `src/components/PlayerStats.tsx` | Aggregated stats from recent matches |
| RankDisplay | `src/components/RankDisplay.tsx` | Tier/LP/win-loss display |
| InGameIndicator | `src/components/InGameIndicator.tsx` | Live game status (in queue / in game) |
| LeaderboardDisplay | `src/components/LeaderboardDisplay.tsx` | Challenger/GM/Master ladder |
| PlatformStatus | `src/components/PlatformStatus.tsx` | Server health indicators |
| PinnedWidget | `src/components/PinnedWidget.tsx` | Compact overlay mode (draggable, top-right) |
| LegalFooter | `src/components/LegalFooter.tsx` | Riot Games legal boilerplate |
| DisplayModeWarning | `src/components/DisplayModeWarning.tsx` | Browser-vs-Tauri detection banner |

### Hooks

| Hook | File | Purpose |
|------|------|---------|
| `useApi` | `src/hooks/useApi.ts` | Shared fetch with `fetchIdRef` + `AbortController` |
| `useTftWatcher` | `src/hooks/useTftWatcher.tsx` | Listens for attach/detach events, auto-resizes overlay |

---

## Data Flow

### Startup Sequence

```
1. main.rs → lib::run()
2. Load .env (project root + data dir)
3. init_database() → SQLite WAL pragma, create tables
4. Print startup banner (version, PID, mode, DB path)
5. tauri::Builder::default()
   a. setup():
      - spawn_watcher(handle, 2000) → polling thread
      - setup_tray(app) → tray icon + menu
   b. manage(AppState) → shared state
   c. generate_handler!() → register 14 commands
6. WebView2 loads dist/index.html
7. React mounts → useEffect detects Tauri vs browser
8. useTftWatcher listens for tft-attached / tft-detached
9. Window visible → user interacts
```

### TFT Auto-Attach Flow

```
                     TFT launches
                         │
                         ▼
    ┌──────────────────────────────┐
    │ process_watcher thread (2s)  │
    │ Win32 FindWindowW("Riot...") │
    │ GetWindowRect → {x,y,w,h}   │
    │ STATE CHANGE detected        │
    │ app_handle.emit("tft-        │
    │   attached", payload)        │
    └──────────────┬───────────────┘
                   │ Tauri event
                   ▼
    ┌──────────────────────────────┐
    │ useTftWatcher hook           │
    │ handleAttached().callbacks   │
    │                              │
    │ snapToGameWindow(info):     │
    │ 1. getCurrentWindow()       │
    │ 2. setPosition(x, y)        │
    │ 3. setSize(width, height)   │
    │ 4. win.show()               │
    └──────────────┬───────────────┘
                   │ Tauri API call
                   ▼
    ┌──────────────────────────────┐
    │ Tauri window resizes +       │
    │ repositions to match game    │
    │ Overlay appears              │
    └──────────────────────────────┘
```

### TFT Detach Flow

```
                    TFT closes
                         │
                         ▼
    ┌──────────────────────────────┐
    │ process_watcher thread       │
    │ FindWindowW returns None     │
    │ STATE CHANGE: attached→None  │
    │ app_handle.emit("tft-        │
    │   detached", payload)        │
    └──────────────┬───────────────┘
                   │
                   ▼
    ┌──────────────────────────────┐
    │ useTftWatcher hook           │
    │ handleDetached()             │
    │ setState("detached")         │
    │ hideOverlay():               │
    │   getCurrentWindow().hide()  │
    └──────────────┬───────────────┘
                   │
                   ▼
          Overlay hidden
     App stays in system tray
     Tray double-click to restore
```

### User Search Flow

```
User types "Player#TAG"
         │
         ▼
┌──────────────────────┐
│ 1. resolve_player()   │
│    invoke("resolve-   │
│    player", name,tag) │
└──────────┬───────────┘
           │ IPC call
           ▼
┌──────────────────────┐
│ 2. Rust backend      │
│    ┌─ Mock mode:     │
│    │  load mock JSON │
│    ├─ Direct mode:   │
│    │  Riot API v1    │
│    │  GET /riot/     │
│    │  account/v1/... │
│    └─ Proxy mode:    │
│       forward to     │
│       proxy.py       │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 3. Cache in SQLite   │
│    INSERT OR REPLACE │
│    INTO players(...) │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 4. Frontend renders  │
│    Rank, Matches,    │
│    Stats, Live game  │
│    status            │
└──────────────────────┘
```

---

## API Architecture

### Three API Modes

| Mode | Config | Behavior | Use Case |
|------|--------|----------|----------|
| **Mock** | No `.env` or `USE_MOCK=true` | Reads local JSON files | Development, demo, offline |
| **Direct** | `RGAPI_KEY=***` | Calls Riot API directly | Production with user key |
| **Proxy** | `RIOT_PROXY_URL=...` | Routes through backend | Browser preview, load balancing |

### Rate Limiting

- **Mock mode**: No limits
- **Direct mode**: Respects Riot's limits (20 req/s, 100 req/2 min)
- **Proxy mode**: Configurable via proxy middleware

---

## Database Schema

### Table: `players`

| Column | Type | Description |
|--------|------|-------------|
| puuid | TEXT PK | Riot PUUID |
| game_name | TEXT | Riot ID name |
| tag_line | TEXT | Riot ID tag |
| summoner_level | INTEGER | Account level |
| last_updated | TEXT ISO8601 | Cache timestamp |

### Table: `matches`

| Column | Type | Description |
|--------|------|-------------|
| match_id | TEXT PK | Match UUID |
| puuid | TEXT FK | Owner's PUUID |
| data | TEXT JSON | Full match payload |
| placement | INTEGER | Final placement (1-8) |
| played_at | TEXT ISO8601 | Match timestamp |
| cached_at | TEXT ISO8601 | Cache timestamp |

---

## Security & Compliance

- **CSP**: locked to self + Riot CDN images + Riot API domains
- **Env isolation**: API key never hardcoded, loaded from `.env` only
- **No augment/legend data**: blocked at `api.rs` — `get_augment_win_rates()` returns `Err`
- **Legal footer**: rendered on every dashboard view
- **Process isolation**: watcher thread uses `FindWindowW` only — no memory scanning

## Build Targets

| Target | Config | Installer |
|--------|--------|-----------|
| Windows x86_64 | `cargo build --target x86_64-pc-windows-gnu --release` | NSIS .exe (currentUser, license page) |
| Linux x86_64 | `cargo build --release` | .deb + .AppImage |
| macOS arm64 | `cargo build --release` | .dmg (minimum 12.0) |
| Raspberry Pi (ARM64) | `cargo build --release` | Dev target (no production bundle) |
