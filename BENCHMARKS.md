# HexForge Companion — Benchmarks

## Build Benchmarks

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| Frontend build time | <3s | **1.48s** (54 modules) | ✅ |
| Frontend dist size | <2MB gzip | **186 KB** total (51.7 KB gzip) | ✅ |
| Rust compile (debug) | <5 min | N/A* | ⏳ |
| Rust compile (release) | <10 min | N/A* | ⏳ |
| Binary size (release) | <15MB | N/A* | ⏳ |
| Rust `cargo check` | — | ✅ (verified in earlier session) | ✅ |

*\*Rust compile benchmarks require running on the target machine. Run:*
```bash
cd src-tauri && time cargo build --release
ls -lh target/release/hexforge-companion
```

## Runtime Benchmarks

### Target: <1% CPU idle, <100MB RAM idle

| Metric | Target | Notes |
|--------|--------|-------|
| Idle CPU usage | <1% | No background threads active; polling thread sleeps 2s between iterations |
| Idle RAM usage | <100MB | Tauri v2 uses ~40-60MB baseline; SQLite cache adds ~5-20MB depending on match history |
| Startup-to-ready time | <1s | Frontend built assets (186 KB) load near-instantly |
| API response (mock) | <5ms | Local JSON file reads |
| API response (direct) | <100ms | Riot API latency (varies by region) |
| Poll interval | 2s | Process watcher checks every 2s for TFT process |

### Measuring on target machine

```bash
# CPU + RAM with pidstat
pidstat -p $(pgrep hexforge-companion) 2 30

# Detailed memory with smem (Linux)
smem -H -P "hexforge"

# Windows tasklist
tasklist /FI "IMAGENAME eq hexforge-companion.exe" /FO CSV
```

## Competitor Comparison

### Overwolf-based Apps (Mobalytics, MetaTFT, TFTactics, TFT Academy)

| Metric | Overwolf Apps | HexForge Companion |
|--------|--------------|-------------------|
| Platform dependency | Requires Overwolf runtime (200MB+) | **Standalone Tauri v2 binary** |
| Binary size | 200MB+ (Overwolf + app) | **<15MB target** |
| Idle CPU usage | 3-8% (Overwolf background service) | **<1%** (sleeping polling thread) |
| Overlay method | Overwolf DLL injection | **Transparent WebView2 overlay** |
| Hotkey | Overwolf platform hotkey | **Ctrl+Shift+H** (dedicated) |
| Mock mode | Requires Riot API key | **Built-in mock mode, no key needed** |
| Lifecycle | Runs as background service | **System tray lifecycle** |
| Privacy | Telemetry, ad profiling | **Zero telemetry by default** |
| GDPR wipe | Account-based deletion | **One-click local wipe** |
| Platform support | Windows only | **Windows, macOS, Linux** |
| Tech stack | HTML+JS in Overwolf Chromium | **Rust + Tauri v2 (native)** |

### Standalone Desktop Apps (DAK.GG / LoLCHESS.GG)

| Metric | DAK.GG | HexForge Companion |
|--------|--------|-------------------|
| Tech stack | Electron (heavy) | **Tauri v2 (lightweight)** |
| Binary size | ~100-150MB | **<15MB target** |
| Game focus | 9 games (LoL, TFT, Valorant, etc.) | **Hyper-focused on TFT** |
| Platform support | Windows + limited macOS | **Windows, macOS, Linux** |
| Privacy | Telemetry collected | **Zero telemetry, one-click wipe** |
| Open source | Closed-source binary | **Open source (MIT)** |
| Mock mode | Requires API key | **Built-in mock mode** |
| TFT-specific UX | Secondary to multi-game focus | **Pure TFT overlay experience** |

### Web-Only (Tactics.tools)

| Metric | Tactics.tools | HexForge Companion |
|--------|--------------|-------------------|
| In-game overlay | None (alt-tab required) | **Transparent overlay on game** |
| Live tracking | Manual refresh | **Auto-attach, auto-polling** |
| Hotkey | N/A | **Ctrl+Shift+H toggle** |

## Compliance Verification

| Requirement | Status | Notes |
|------------|--------|-------|
| No augment/legend win rates | ✅ Blocked at API client level (`api.rs`) |
| No live scouting | ✅ Spectator data never used or stored |
| No direct decision dictation | ✅ Displays data, never tells user what to play |
| Legal boilerplate | ✅ `LegalFooter.tsx` on every dashboard |
| GDPR one-click wipe | ✅ `request_account_deletion` command |
| Process detection only | ✅ `FindWindowW`/`CreateToolhelp32Snapshot` only — no memory scanning |

## Size Budget

| Component | Current | Target |
|-----------|---------|--------|
| Frontend (dist/) | **186 KB** | <2 MB |
| Rust binary (release) | N/A* | <15 MB |
| NSIS installer | N/A* | <20 MB |
| Installer + data on disk | N/A* | <50 MB |

*\*Measurable only after release build on target machine.*
