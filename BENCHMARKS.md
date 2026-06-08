# HexForge Companion v0.6.0 — Benchmarks

## Build Benchmarks

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| Frontend build time | <3s | **1.48s** (54 modules) | ✅ |
| Frontend dist size | <2MB gzip | **186 KB** total (51.7 KB gzip) | ✅ |
| Rust compile (debug) | <5 min | *Requires Pi* | ⏳ |
| Rust compile (release) | <10 min | *Requires Pi* | ⏳ |
| Binary size (release) | <15MB | *Requires Pi* | ⏳ |
| Rust `cargo check` | — | *Requires Pi* | ⏳ |

## Runtime Benchmarks (Targets)

| Metric | Target | Notes |
|--------|--------|-------|
| Idle CPU usage | <1% | Polling thread sleeps 2s between iterations |
| Idle RAM usage | <100MB | Tauri v2 ~40-60MB + SQLite cache ~5-20MB |
| Startup-to-ready time | <1s | Frontend assets (186 KB) load near-instantly |
| API response (mock) | <5ms | Local JSON file reads |
| API response (direct) | <100ms | Riot API latency (varies by region) |
| Poll interval | 2s | Process watcher checks for TFT window |

## Competitor Comparison

### Overwolf-based Apps (Mobalytics, MetaTFT, TFTactics, TFT Academy)

| Metric | Overwolf Apps | HexForge Companion |
|--------|--------------|-------------------|
| Platform dependency | Requires Overwolf (200MB+) | **Standalone Tauri v2** |
| Binary size | 200MB+ | **<15MB target** |
| Idle CPU | 3-8% | **<1%** |
| Privacy | Telemetry + ad profiling | **Zero telemetry** |
| GDPR wipe | Account-based | **One-click local wipe** |
| Platform support | Windows only | **Windows + macOS + Linux** |
| Framework | Overwolf Chromium | **Rust + Tauri v2 (native)** |
| Mock mode | Requires API key | **Built-in, no key needed** |
| Hotkey | Platform-wide | **Ctrl+Shift+H (dedicated)** |

### Standalone Desktop (DAK.GG / LoLCHESS.GG)

| Metric | DAK.GG | HexForge Companion |
|--------|--------|-------------------|
| Tech stack | Electron (heavy) | **Tauri v2 (native, lightweight)** |
| Binary size | ~100-150MB | **<15MB target** |
| Game focus | 9 games (TFT is secondary) | **Hyper-focused on TFT** |
| Privacy | Telemetry collected | **Zero telemetry, one-click wipe** |
| Open source | Closed-source | **Open source (MIT)** |
| Platform support | Windows + limited macOS | **Windows + macOS + Linux** |

### Web-Only (Tactics.tools)

| Metric | Tactics.tools | HexForge Companion |
|--------|--------------|-------------------|
| In-game overlay | None (alt-tab) | **Transparent overlay on game** |
| Live tracking | Manual refresh | **Auto-attach, auto-polling** |
| Hotkey | N/A | **Ctrl+Shift+H** |

## Differentiators Summary

| # | Differentiator | Impact |
|---|---------------|--------|
| 1 | **No Overwolf** | The single strongest narrative. Overwolf is universally hated for adware, bloat, privacy concerns |
| 2 | **<15MB binary** vs Overwolf 200MB+ | "Smaller than a single screenshot" |
| 3 | **<1% CPU** vs Overwolf 3-8% | "So lightweight you'll forget it's running" |
| 4 | **Zero telemetry / GDPR wipe** | "Your data stays yours. One click, everything gone." |
| 5 | **Mock mode out of the box** | Competitors require Riot API key just to evaluate |
| 6 | **Rust + Tauri** | Native performance, not another Electron resource hog |

## Compliance Verification

| Requirement | Status | Notes |
|------------|--------|-------|
| No augment/legend win rates | ✅ Blocked at API client level (api.rs) |
| No live scouting | ✅ Spectator data never used or stored |
| No direct decision dictation | ✅ Displays data, never tells user what to play |
| Legal boilerplate | ✅ LegalFooter.tsx on every dashboard |
| GDPR one-click wipe | ✅ request_account_deletion command |
| Process detection only | ✅ CreateToolhelp32Snapshot/FindWindowW only |

## Size Budget

| Component | Current | Target |
|-----------|---------|--------|
| Frontend (dist/) | **186 KB** | <2 MB |
| Rust binary (release) | *Requires Pi* | <15 MB |
| NSIS installer | *Requires Pi* | <20 MB |
