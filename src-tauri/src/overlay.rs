use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;

/// Hit-test rectangle for the overlay panel (in physical pixels).
/// Clicks inside this rect are captured by the overlay.
/// Clicks outside pass through to the game. Same pattern as LoLProxChat.
static PANEL_HIT_RECT: Mutex<(u32, u32)> = Mutex::new((320, 400));

/// Set whether cursor events pass through the overlay window.
/// Used by hud_bounds_enter/hud_bounds_leave from frontend.
pub fn set_passthrough(window: &tauri::Window, enabled: bool) {
    let _ = window.set_ignore_cursor_events(enabled);
}

/// Start a hit-test polling loop on a spawned thread.
/// Checks cursor position every ~33ms (30 Hz) and toggles
/// `set_ignore_cursor_events` so clicks outside panel widgets
/// fall through to the game. Same pattern as LoLProxChat.
pub fn spawn_hit_test_loop(window: tauri::WebviewWindow) {
    std::thread::spawn(move || {
        let mut last_ignore = true;
        let interval = Duration::from_millis(33);

        loop {
            let (pw, ph) = *PANEL_HIT_RECT.lock().unwrap();

            // Get cursor position in screen coordinates
            let cursor = match get_cursor_pos() {
                Some(p) => p,
                None => {
                    std::thread::sleep(interval);
                    continue;
                }
            };

            // Get overlay window position in screen coordinates
            let win_pos = match window.outer_position().ok() {
                Some(p) => (p.x, p.y),
                None => {
                    std::thread::sleep(interval);
                    continue;
                }
            };

            // Check if cursor is inside the panel hit area
            let over_panel = cursor.0 >= win_pos.0
                && cursor.0 < win_pos.0 + pw as i32
                && cursor.1 >= win_pos.1
                && cursor.1 < win_pos.1 + ph as i32;

            // Toggle passthrough based on cursor position
            let should_ignore = !over_panel;
            if should_ignore != last_ignore {
                let _ = window.set_ignore_cursor_events(should_ignore);
                last_ignore = should_ignore;
            }

            std::thread::sleep(interval);
        }
    });
}

/// Update the hit-test rectangle dimensions.
/// Called from frontend when panel size changes.
pub fn set_panel_size(width: u32, height: u32) {
    let clamped_w = width.clamp(100, 4000);
    let clamped_h = height.clamp(100, 4000);
    let mut rect = PANEL_HIT_RECT.lock().unwrap();
    rect.0 = clamped_w;
    rect.1 = clamped_h;
}

/// Get current panel hit-test size.
pub fn get_panel_size() -> (u32, u32) {
    *PANEL_HIT_RECT.lock().unwrap()
}

// ── Win32 cursor helper ──────────────────────────────────

#[cfg(target_os = "windows")]
fn get_cursor_pos() -> Option<(i32, i32)> {
    unsafe {
        let mut pt = windows::Win32::Foundation::POINT::default();
        if windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt).is_ok() {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_cursor_pos() -> Option<(i32, i32)> {
    None
}