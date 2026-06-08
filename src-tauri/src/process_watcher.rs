use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
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

// ────────────────────────────────────────────────────────────
//  Platform-specific window detection
// ────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use super::TftState;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::MonitorFromWindow;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowRect, IsWindowVisible, GetWindowTextLengthW,
        GetWindowTextW, IsIconic, MONITOR_DEFAULTTONULL,
    };

    /// Window class names that match the TFT / League in-game window.
    const TFT_WINDOW_CLASSES: &[&str] = &[
        "RiotWindowClass",         // LoL/TFT actual game window
        "RCLIENT",                 // Riot Client (pre-game lobby)
    ];

    /// Window title substrings that indicate TFT.
    const TFT_TITLE_MARKS: &[&str] = &[
        "League of Legends",
        "Teamfight Tactics",
        "TFT",
    ];

    /// Check if a single HWND matches the TFT process.
    unsafe fn is_tft_window(hwnd: HWND) -> bool {
        // Must be visible
        if IsWindowVisible(hwnd).is_err() {
            return false;
        }

        // Check class name
        for class_name in TFT_WINDOW_CLASSES {
            let wide: Vec<u16> = class_name.encode_utf16().chain(std::iter::once(0)).collect();
            let found = FindWindowW(Some(&wide[0] as *const u16 as _), None);
            if found == hwnd {
                return true;
            }
        }

        // Check window title for TFT keywords
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buf = vec![0u16; (len as usize + 1)];
            let written = GetWindowTextW(hwnd, &mut buf);
            if written > 0 {
                let title = String::from_utf16_lossy(&buf[..written as usize]);
                for mark in TFT_TITLE_MARKS {
                    if title.contains(mark) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Find the TFT window and return its geometry.
    /// Returns `None` if no TFT window is visible.
    pub fn find_tft_window() -> Option<TftState> {
        unsafe {
            // Try the primary known class first (fast path)
            let wide: Vec<u16> = b"RiotWindowClass\0".iter().map(|&c| c as u16).collect();
            let hwnd = FindWindowW(Some(&wide[0] as *const u16 as _), None);
            if hwnd != HWND(std::ptr::null_mut()) && IsWindowVisible(hwnd).is_ok() && !IsIconic(hwnd).as_bool() {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    return Some(TftState::Attached {
                        x: rect.left,
                        y: rect.top,
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    });
                }
            }

            // Fallback: search via RCLIENT class
            let wide2: Vec<u16> = b"RCLIENT\0".iter().map(|&c| c as u16).collect();
            let hwnd2 = FindWindowW(Some(&wide2[0] as *const u16 as _), None);
            if hwnd2 != HWND(std::ptr::null_mut()) && IsWindowVisible(hwnd2).is_ok() {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd2, &mut rect).is_ok() {
                    return Some(TftState::Attached {
                        x: rect.left,
                        y: rect.top,
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    });
                }
            }

            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::TftState;

    /// On non-Windows platforms, stub out detection.
    /// The overlay can still be positioned manually or via env vars.
    pub fn find_tft_window() -> Option<TftState> {
        // Check env vars for manual positioning (dev/debug)
        if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
            std::env::var("TFT_X"),
            std::env::var("TFT_Y"),
            std::env::var("TFT_W"),
            std::env::var("TFT_H"),
        ) {
            return Some(TftState::Attached {
                x: x.parse().unwrap_or(0),
                y: y.parse().unwrap_or(0),
                width: w.parse().unwrap_or(1920),
                height: h.parse().unwrap_or(1080),
            });
        }

        // On Linux dev (Pi 5): always report attached at full-screen
        // so the overlay shows during development.
        if let Ok(display) = std::env::var("DISPLAY") {
            if !display.is_empty() {
                return Some(TftState::Attached {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                });
            }
        }

        None
    }
}

// ────────────────────────────────────────────────────────────
//  Watcher thread
// ────────────────────────────────────────────────────────────

/// Spawns a background thread that polls for the TFT window every
/// `interval_ms` milliseconds and emits Tauri events on state changes.
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

        eprintln!(
            "[HexForge::Watcher] Started — polling every {}ms",
            interval_ms
        );

        while running_clone.load(Ordering::Relaxed) {
            let current = platform::find_tft_window();

            // Only emit on state transition
            if current != last_state {
                let payload = TftStatePayload {
                    state: current.unwrap_or(TftState::Detached),
                };

                match &current {
                    Some(TftState::Attached { x, y, width, height }) => {
                        eprintln!(
                            "[HexForge::Watcher] TFT attached — window at ({},{}) {}x{}",
                            x, y, width, height
                        );
                        let _ = app_handle.emit("tft-attached", &payload);
                    }
                    None | Some(TftState::Detached) => {
                        eprintln!("[HexForge::Watcher] TFT detached");
                        let _ = app_handle.emit("tft-detached", &payload);
                    }
                }

                last_state = current;
            }

            thread::sleep(interval);
        }

        eprintln!("[HexForge::Watcher] Stopped");
    });

    running
}

/// Convenience: spawn the watcher on the Tauri `setup` hook.
/// Call from `lib.rs` or `main.rs` setup closure.
#[allow(dead_code)]
pub fn setup_watcher(app: &AppHandle) -> Arc<AtomicBool> {
    spawn_watcher(app.clone(), 2000)
}