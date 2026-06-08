# HexForge Companion v0.5.0 — Release Checklist

## Preflight

- [ ] **Version check**: Verify `package.json`, `Cargo.toml`, `tauri.conf.json` all show `0.5.0`
- [ ] **Changelog**: Document notable changes since last release
- [ ] **LICENSE**: MIT file present in repo root
- [ ] **API key**: Dev key refreshed at `developer.riotgames.com` (expires 24h)
- [ ] **Environment**: `.env` file exists with valid `RGAPI_KEY` for production testing
- [ ] **Git status**: Working tree clean, all changes committed
- [ ] **BENCHMARKS.md**: Updated with latest measurements

## Build

- [ ] **Frontend**: `npm run build` exits 0 cleanly
  ```bash
  cd ~/projects/hexforge-companion && npm run build
  ```
- [ ] **Rust check**: `cargo check` exits 0 with no warnings
  ```bash
  cd src-tauri && cargo check 2>&1
  ```
- [ ] **Rust build (release)**: `cargo build --release` exits 0
  ```bash
  cd src-tauri && cargo build --release
  ```
- [ ] **Binary size**: `ls -lh target/release/hexforge-companion` — target <15MB
- [ ] **Windows cross-compile** (if building on Linux):
  ```bash
  cd src-tauri && cargo build --target x86_64-pc-windows-gnu --release
  ```
- [ ] **NSIS installer** (Windows only):
  ```bash
  cd src-tauri && cargo tauri build --bundles nsis --target x86_64-pc-windows-gnu
  ```

## Test

- [ ] **Launch test**: App opens without errors
- [ ] **Overlay window**: Transparent, frameless, always-on-top, skip-taskbar
- [ ] **Mock mode**: Search any player name — mock data appears
- [ ] **Direct mode**: Set `RGAPI_KEY` in `.env` — real API data works
- [ ] **Process watcher**: TFT attach/detach events fire correctly
- [ ] **Auto-resize**: Overlay snaps to TFT window geometry on attach
- [ ] **Auto-hide**: Overlay hides when TFT closes
- [ ] **Tray icon**: System tray icon registers; show/hide/quit works
- [ ] **Hotkey**: Ctrl+Shift+H toggles overlay visibility
- [ ] **Pinned widget**: Pin button works, 30s auto-poll
- [ ] **Refresh**: Refresh Matches button works, spinner shows
- [ ] **Leaderboard**: Challenger/GM/Master data displays
- [ ] **Platform status**: Server health indicators work
- [ ] **Error handling**: Invalid summoner shows error message
- [ ] **Legal footer**: Riot boilerplate visible on every screen
- [ ] **PUUID clear**: After 30s of TFT detach, player state resets
- [ ] **GDPR wipe**: `request_account_deletion` command works
- [ ] **Update check**: Startup triggers updater check (no crash if no update)

## Compliance

- [ ] **No augment win rates**: Verify `get_augment_win_rates()` returns `Err`
- [ ] **No live scouting**: Spectator endpoint not called
- [ ] **No dictation**: App never tells user what to play
- [ ] **Legal boilerplate**: Present on every dashboard view
- [ ] **GDPR wipe**: One-click account data deletion
- [ ] **CSP**: Check `tauri.conf.json` `security.csp`
- [ ] **Process safety**: No memory/DLL injection — only `FindWindowW`/`CreateToolhelp32Snapshot`

## Installer Test (NSIS)

- [ ] **Clean install**: Install on bare Windows machine (or VM)
- [ ] **No overwrite**: Existing `.env` preserved on reinstall
- [ ] **Uninstall**: Removes `%LOCALAPPDATA%/HexForge/` directory
- [ ] **Desktop shortcut**: Created (if configured)
- [ ] **Start menu**: Folder created under `HexForge Companion`
- [ ] **WebView2**: Fallback download works if not pre-installed

## Deploy

- [ ] **Tag release**:
  ```bash
  git tag -a v0.5.0 -m "HexForge Companion v0.5.0"
  git push origin v0.5.0
  ```
- [ ] **GitHub Release**: Create from tag, attach installer files
- [ ] **CI trigger** (if configured): Push to `release` branch triggers workflow
- [ ] **Update manifest**: Verify `latest.json` uploaded to release assets
- [ ] **Download page**: `http://raspberrypi.local:1421/download/` serves latest .exe

## Post-Release

- [ ] **Monitor issues**: Check GitHub Issues for crash reports
- [ ] **Refresh API key**: Dev key expires — set up Production key via Riot review
- [ ] **Announce**: Share in relevant Discord/Twitter/Reddit communities
- [ ] **Docs**: Update `docs/API.md`, `docs/COMPLIANCE.md` if needed
