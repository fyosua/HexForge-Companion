use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use crate::hlog;

/// Whether the TFT game process/window is currently detected.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum TftState {
    Detached,
    Attached { x: i32, y: i32, width: i32, height: i32 },
}

#[derive(Debug, Clone, Serialize)]
pub struct TftStatePayload {
    pub state: TftState,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClearPuuidPayload {
    pub reason: &'static str,
}

/// Check if a TFT/League process is running on this platform.
/// Returns `true` if any matching process is found.
fn is_tft_process_running() -> bool {
    use std::ffi::OsStr;
    use sysinfo::System;
    let system = System::new_all();
    system.processes_by_name(OsStr::new("League of Legends.exe")).count() > 0
}

// ── Windows window geometry ──────────────────────────────

#[cfg(target_os = "windows")]
fn get_tft_geometry() -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowRect, IsWindowVisible, IsIconic,
    };
    use windows::core::PCWSTR;

    unsafe {
        // Primary: RiotWindowClass (TFT in-game)
        let class_name: Vec<u16> = "RiotWindowClass\0".encode_utf16().collect();
        let hwnd = FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR(std::ptr::null()));
        if let Ok(w) = hwnd {
            if IsWindowVisible(w).as_bool() && !IsIconic(w).as_bool() {
                let mut rect = windows::Win32::Foundation::RECT::default();
                if GetWindowRect(w, &mut rect).is_ok() {
                    return Some((
                        rect.left, rect.top,
                        rect.right - rect.left, rect.bottom - rect.top,
                    ));
                }
            }
        }

        // Fallback: RCLIENT (Riot Client lobby)
        let class_name2: Vec<u16> = "RCLIENT\0".encode_utf16().collect();
        let hwnd2 = FindWindowW(PCWSTR(class_name2.as_ptr()), PCWSTR(std::ptr::null()));
        if let Ok(w) = hwnd2 {
            if IsWindowVisible(w).as_bool() {
                let mut rect = windows::Win32::Foundation::RECT::default();
                if GetWindowRect(w, &mut rect).is_ok() {
                    return Some((
                        rect.left, rect.top,
                        rect.right - rect.left, rect.bottom - rect.top,
                    ));
                }
            }
        }

        None
    }
}

// ── Non-Windows geometry fallback ────────────────────────

#[cfg(not(target_os = "windows"))]
fn get_tft_geometry() -> Option<(i32, i32, i32, i32)> {
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
    Some((0, 0, 1920, 1080))
}

// ── Watcher thread ───────────────────────────────────────

pub fn spawn_watcher(app_handle: AppHandle, interval_ms: u64) -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    thread::spawn(move || {
        let interval = Duration::from_millis(interval_ms);
        let mut last_process_running: Option<bool> = None;
        let mut detach_since: Option<Instant> = None;
        let mut puuid_clear_emitted = false;
        let mut poll_count: u64 = 0;

        eprintln!("[HexForge::Watcher] Started — polling every {}ms", interval_ms);
        hlog!("Watcher started — polling every {}ms", interval_ms);

        while running_clone.load(Ordering::Relaxed) {
            poll_count += 1;
            let process_running = is_tft_process_running();
            let geometry = if process_running { get_tft_geometry() } else { None };

            // Log every 10th poll + every state change (verbose for debugging)
            if poll_count % 10 == 0 || last_process_running != Some(process_running) {
                hlog!("Watcher poll#{} — process_running={}, geometry={:?}", poll_count, process_running,
                    geometry.map(|(x,y,w,h)| format!("({},{}){}x{}",x,y,w,h)));
            }

            // 🚫 NO window toggling from watcher — emit events only
            // Frontend uses these events to update UI and user clicks "Launch Overlay"
            if last_process_running != Some(process_running) {
                if process_running {
                    hlog!("Watcher — process DETECTED (emitting tft-attached)");
                    let payload = TftStatePayload {
                        state: TftState::Attached {
                            x: geometry.map(|g| g.0).unwrap_or(0),
                            y: geometry.map(|g| g.1).unwrap_or(0),
                            width: geometry.map(|g| g.2).unwrap_or(1920),
                            height: geometry.map(|g| g.3).unwrap_or(1080),
                        },
                    };
                    let _ = app_handle.emit("tft-attached", &payload);
                    detach_since = None;
                    puuid_clear_emitted = false;
                } else {
                    hlog!("Watcher — process LOST (emitting tft-detached)");
                    let payload = TftStatePayload { state: TftState::Detached };
                    let _ = app_handle.emit("tft-detached", &payload);
                    detach_since = Some(Instant::now());
                    puuid_clear_emitted = true;
                }
                last_process_running = Some(process_running);
            }

            // Emit clear-puuid after 30s of being detached
            if let Some(since) = detach_since {
                if since.elapsed() >= Duration::from_secs(30) && !puuid_clear_emitted {
                    let clear_payload = ClearPuuidPayload { reason: "TFT closed for 30s" };
                    let _ = app_handle.emit("clear-puuid", &clear_payload);
                    puuid_clear_emitted = true;
                    hlog!("Watcher — PUUID cleared (TFT gone for 30s)");
                }
            }

            thread::sleep(interval);
        }

        eprintln!("[HexForge::Watcher] Stopped");
        hlog!("Watcher stopped — {} total polls", poll_count);
    });

    running
}