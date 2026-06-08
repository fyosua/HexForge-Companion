pub fn set_passthrough(window: &tauri::Window, enabled: bool) {
    let _ = window.set_ignore_cursor_events(enabled);
}

/// Initialize the overlay cursor passthrough system.
/// Called once at app startup.
pub fn init_overlay(window: &tauri::Window) {
    // Start in passthrough mode — clicks fall through to the game
    set_passthrough(window, true);

    // Position overlay on the primary monitor
    if let Some(monitor) = window.primary_monitor().unwrap() {
        let size = monitor.size();
        let _ = window.set_size(tauri::PhysicalSize::new(size.width, size.height));
        let _ = window.set_position(tauri::PhysicalPosition::new(0, 0));
    }
}