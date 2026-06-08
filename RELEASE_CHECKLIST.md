# HexForge Companion v0.6.0 — Release Checklist

## Preflight

- [ ] **Version check**: package.json, Cargo.toml, tauri.conf.json all show `0.6.0`
- [ ] **Signing keys**: `~/.tauri/hexforge.key` exists with password
- [ ] **Signing keys**: `tauri.conf.json` plugins.updater.pubkey matches key.pub
- [ ] **Git status**: Working tree clean, all changes committed
- [ ] **Frontend**: `npm run build` exits 0

## Build

- [ ] **Rust check**: `cargo check` exits 0 with no warnings
  ```bash
  cd src-tauri && cargo check 2>&1
  ```
- [ ] **Release build**: `cargo build --release` exits 0
  ```bash
  cd src-tauri && cargo build --release
  ```
- [ ] **Binary size**: `ls -lh target/release/hexforge-companion` — target <15MB
- [ ] **Windows cross-compile**:
  ```bash
  cd src-tauri && cargo build --target x86_64-pc-windows-gnu --release
  ```
- [ ] **NSIS installer**:
  ```bash
  cd src-tauri && cargo tauri build --bundles nsis --target x86_64-pc-windows-gnu
  ```
- [ ] **GitHub Actions (CI)**: Push tag v0.6.0 triggers release.yml

## Test

- [ ] **Dual-window**: Dashboard opens with title bar; overlay hidden
- [ ] **TFT attach**: Overlay appears, dashboard hides, window snaps to game
- [ ] **TFT detach**: Dashboard reappears, overlay hides
- [ ] **Tray icon**: Show/hide/quit works
- [ ] **Hotkey**: Ctrl+Shift+H toggles overlay
- [ ] **Mock mode**: Search any name — data appears
- [ ] **Direct mode**: RGAPI_KEY in .env — real API data works
- [ ] **Pinned widget**: Pin/unpin + 30s auto-poll
- [ ] **Refresh**: Refresh button works
- [ ] **Leaderboard**: Challenger/GM/Master displays
- [ ] **Platform status**: Server health indicators
- [ ] **GDPR wipe**: request_account_deletion works
- [ ] **Update check**: Checks on startup (no crash if no update)

## Compliance

- [ ] No augment/legend win rates displayed
- [ ] No live scouting (spectator endpoint not called)
- [ ] No direct decision dictation
- [ ] Legal boilerplate on every dashboard
- [ ] GDPR one-click wipe
- [ ] Read-only process detection (CreateToolhelp32Snapshot only)

## Deploy

- [ ] **Tag release**: `git tag v0.6.0 && git push origin v0.6.0`
- [ ] **CI run**: Verify release.yml completes on GitHub Actions
- [ ] **Download**: verify .exe available at /download/ on proxy
- [ ] **Landing page**: http://raspberrypi.local:1421/ looks correct
- [ ] **Release notes**: Update GitHub Release with changelog

## Post-Release

- [ ] Refresh Riot API dev key (expires 24h)
- [ ] Submit for Production API key via Riot review portal
- [ ] Announce in relevant communities
