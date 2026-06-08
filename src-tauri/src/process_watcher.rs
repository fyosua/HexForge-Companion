use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager, {AppHandle, Emitter, Manager};
use serde::Serialize;

/// Whether the TFT game process/window is currently detected.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum TftState {
    /// TFT is not running (or not in-game).
    Detached,
    /// TFT is running and the overlay can attach.
    Attached {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
}

/// Payload sent to the frontend when TFT state changes.
#[derive(Debug, Clone, Serialize)]
pub struct TftStatePayload {
    pub state: TftState,
}

/// Payload sent after a grace period to clear cached PUUID.
#[derive(Debug, Clone, Serialize)]
pub struct ClearPuuidPayload {
    pub reason: &'static str,
}

// ────────────────────────────────────────────────────────────
//  Platform-specific TFT process detection
// ────────────────────────────────────────────────────────────

/// Check if a TFT/League process is running on this platform.
/// Returns `true` if any matching process is found.
#[cfg(target_os = "windows")]
fn is_tft_process_running() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
        PROCESSENTRY32W,
    };
    use windows::Win32::Storage::FileSystem::MAX_PATH;
    use std::ptr::null_mut;

    const TFT_PROCESS_NAMES: &[&str] = &[
        "League of Legends.exe",
        "LeagueClient.exe",
        "RiotClientServices.exe",
    ];

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_invalid() {
            return false;
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0u16; MAX_PATH as usize],
        };

        if Process32FirstW(snapshot, &mut entry).is_err() {
            let _ = windows::Win32::Foundation::CloseHandle(snapshot);
            return false;
        }

        loop {
            let exe_name = String::from_utf16_lossy(
                &entry.szExeFile[..entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0)],
            );

            for name in TFT_PROCESS_NAMES {
                if exe_name.eq_ignore_ascii_case(name) {
                    let _ = windows::Win32::Foundation::CloseHandle(snapshot);
                    return true;
                }
            }

            if Process32NextW(snapshot, &mut entry).is_err() {
                break;
            }
        }

        let _ = windows::Win32::Foundation::CloseHandle(snapshot);
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn is_tft_process_running() -> bool {
    // Linux: scan /proc for process names
    // Open /proc and check comm/cmdline entries
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let pid_str = entry.file_name().to_string_lossy().to_string();
            if !pid_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            // Read comm file (short process name)
            let comm_path = entry.path().join("comm");
            if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                let comm = comm.trim();
                for name in &["league", "lol", "riot", "tft"] {
                    if comm.to_ascii_lowercase().contains(name) {
                        return true;
                    }
                }
            }

            // Read cmdline (full command line)
            let cmdline_path = entry.path().join("cmdline");
            if let Ok(cmdline) = std::fs::read_to_string(&cmdline_path) {
                let cmdline = cmdline.replace('\0', " ");
                for name in &["LeagueClient.exe", "League of Legends.exe"] {
                    if cmdline.contains(name) {
                        return true;
                    }
                }
            }
        }
    }

    // Fallback for dev (Pi): always attached so overlay shows during dev
    if std::env::var("DISPLAY").ok().filter(|d| !d.is_empty()).is_some() {
        return true;
    }

    false
}

/// Get the game window geometry.
/// On Windows, finds the actual TFT window. On other platforms,
/// returns full-screen dimensions from env vars or DISPLAY.
#[cfg(target_os = "windows")]
fn get_tft_geometry() -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowRect, IsWindowVisible, IsIconic,
    };

    unsafe {
        // Primary: RiotWindowClass (TFT in-game)
        let wide: Vec<u16> = b"RiotWindowClass\0".iter().map(|&c| c as u16).collect();
        let hwnd = FindWindowW(Some(&wide[0] as *const u16 as _), None);
        if hwnd != HWND(std::ptr::null_mut()) && IsWindowVisible(hwnd).is_ok() && !IsIconic(hwnd).as_bool() {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                return Some((rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top));
            }
        }

        // Fallback: RCLIENT (Riot Client lobby)
        let wide2: Vec<u16> = b"RCLIENT\0".iter().map(|&c| c as u16).collect();
        let hwnd2 = FindWindowW(Some(&wide2[0] as *const u16 as _), None);
        if hwnd2 != HWND(std::ptr::null_mut()) && IsWindowVisible(hwnd2).is_ok() {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd2, &mut rect).is_ok() {
                return Some((rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top));
            }
        }

        None
    }
}

#[cfg(not(target_os = "windows"))]
fn get_tft_geometry() -> Option<(i32, i32, i32, i32)> {
    // Env vars for manual positioning
    if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
        std::env::var("TFT_X"),
        std::env::var("TFT_Y"),
        std::env::var("TFT_W"),
        std::env::var("TFT_H"),
    ) {
        return Some((
            x.parse().unwrap_or(0),
            y.parse().unwrap_or(0),
            w.parse().unwrap_or(1920),
            h.parse().unwrap_or(1080),
        ));
    }

    // Full-screen default
    Some((0, 0, 1920, 1080))
}

// ────────────────────────────────────────────────────────────
//  Watcher thread
// ────────────────────────────────────────────────────────────

/// Spawns a background thread that polls for the TFT process every
/// `interval_ms` milliseconds and emits Tauri events on state changes.
///
/// Behavior:
/// - On TFT process detected: emits `tft-attached` with window geometry
/// - On TFT process lost: emits `tft-detached`, then after 30s emits
///   `clear-puuid` so the frontend can clear cached player data
///
/// Returns a handle that can be used to stop the watcher.
pub fn spawn_watcher(
    app_handle: AppHandle,
    interval_ms: u64,
) -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    thread::spawn(move || {
        let interval = Duration::from_millis(interval_ms);
        let mut last_state: Option<TftState> = None;
        let mut detach_since: Option<Instant> = None;
        let mut puuid_clear_emitted = false;

        eprintln!(
            "[HexForge::Watcher] Started \u{2014} polling every {}ms",
            interval_ms
        );

        while running_clone.load(Ordering::Relaxed) {
            let process_running = is_tft_process_running();
            let geometry = if process_running {
                get_tft_geometry()
            } else {
                None
            };

            let current = geometry.map(|(x, y, w, h)| TftState::Attached { x, y, width: w, height: h });

            // Only emit on state transition
            if current != last_state {
                let payload = TftStatePayload {
                    state: current.unwrap_or(TftState::Detached),
                };

                match &current {
                    Some(TftState::Attached { x, y, width, height }) => {
                        eprintln!(
                            "[HexForge::Watcher] TFT attached \u{2014} window at ({},{}) {}x{}",
                            x, y, width, height
                        );
                        // Switch to overlay mode: show overlay, hide dashboard
                        if let Some(overlay) = app_handle.get_webview_window("overlay") {
                            let _ = overlay.show();
                        }
                        if let Some(dashboard) = app_handle.get_webview_window("dashboard") {
                            let _ = dashboard.hide();
                        }
                        let _ = app_handle.emit("tft-attached", &payload);
                        detach_since = None;
                        puuid_clear_emitted = false;
                    }
                    None | Some(TftState::Detached) => {
                        if !puuid_clear_emitted {
                            eprintln!("[HexForge::Watcher] TFT detached");
                            // Switch to dashboard mode: hide overlay, show dashboard
                            if let Some(overlay) = app_handle.get_webview_window("overlay") {
                                let _ = overlay.hide();
                            }
                            if let Some(dashboard) = app_handle.get_webview_window("dashboard") {
                                let _ = dashboard.show();
                                let _ = dashboard.set_focus();
                            }
                            let _ = app_handle.emit("tft-detached", &payload);
                            detach_since = Some(Instant::now());
                            puuid_clear_emitted = true;
                        }
                    }

                }
                last_state = current;
            }

            // Emit clear-puuid after 30s of being detached
            if let Some(since) = detach_since {
                if since.elapsed() >= Duration::from_secs(30) && !puuid_clear_emitted {
                    let clear_payload = ClearPuuidPayload { reason: "TFT closed for 30s" };
                    let _ = app_handle.emit("clear-puuid", &clear_payload);
                    puuid_clear_emitted = true;
                    eprintln!("[HexForge::Watcher] PUUID cleared (TFT gone for 30s)");
                }
            }

            thread::sleep(interval);
        }

        eprintln!("[HexForge::Watcher] Stopped");
    });

    running
}
