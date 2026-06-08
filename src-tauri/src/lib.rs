use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

mod api;
mod db;
mod overlay;
mod commands;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub api_key: String,
    pub api_mode: api::ApiMode,
    pub active_puuid: Mutex<Option<String>>,
}

/// Print a formatted startup banner with version, mode, and paths.
fn print_startup_banner(api_mode: &api::ApiMode, db_path: &std::path::Path) {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let separator = "=".repeat(52);

    eprintln!("\n{}", separator);
    eprintln!("  {} v{}", name, version);
    eprintln!("  PID: {}", std::process::id());
    eprintln!("  DB:  {}", db_path.display());
    eprintln!("{}", separator);

    match api_mode {
        api::ApiMode::Mock => {
            eprintln!("  MODE: Mock (offline) — no API calls");
            eprintln!("  Set RGAPI_KEY in .env for live data");
        }
        api::ApiMode::Direct { region, platform, .. } => {
            eprintln!("  MODE: Direct — live Riot API");
            eprintln!("     region:   {}", region);
            eprintln!("     platform: {}", platform);
        }
        api::ApiMode::Proxy { proxy_base } => {
            eprintln!("  MODE: Proxy — routed through backend");
            eprintln!("     proxy: {}", proxy_base);
        }
    }
    eprintln!("{}\n", separator);
}

/// Warn when a release build has no API key configured.
fn warn_if_production_mock(api_mode: &api::ApiMode) {
    if *api_mode == api::ApiMode::Mock && !cfg!(debug_assertions) {
        eprintln!(
            "[HexForge] RELEASE BUILD running in MOCK mode — no RGAPI_KEY configured.\n\
             [HexForge]    Create a .env file with RGAPI_KEY=*** for live data, or\n\
             [HexForge]    set USE_MOCK=true in .env to silence this warning."
        );
    }
}

/// Show overlay window, hide dashboard — called on TFT attach.
pub fn show_overlay(handle: &tauri::AppHandle) {
    if let Some(overlay) = handle.get_webview_window("overlay") {
        let _ = overlay.show();
    }
    if let Some(dashboard) = handle.get_webview_window("dashboard") {
        let _ = dashboard.hide();
    }
}

/// Hide overlay window, show dashboard — called on TFT detach.
pub fn show_dashboard(handle: &tauri::AppHandle) {
    if let Some(overlay) = handle.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
    if let Some(dashboard) = handle.get_webview_window("dashboard") {
        let _ = dashboard.show();
        let _ = dashboard.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file from project root (dev) or app data dir (prod)
    dotenvy::dotenv().ok();
    dotenvy::from_path_override(
        dirs_data_local()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("HexForge")
            .join(".env"),
    ).ok();

    let api_mode = api::ApiMode::from_env();
    let api_key = match &api_mode {
        api::ApiMode::Direct { api_key, .. } => api_key.clone(),
        api::ApiMode::Mock => String::from("MOCK_MODE"),
        api::ApiMode::Proxy { proxy_base } => format!("proxy:{}", proxy_base),
    };

    let db_path = dirs_data_local()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("HexForge")
        .join("db")
        .join("storage.db");

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("create HexForge db directory");
    }

    let conn = db::init_database(
        db_path.to_str().expect("valid db path"),
    ).expect("initialize SQLite database");

    // Print startup banner
    print_startup_banner(&api_mode, &db_path);
    warn_if_production_mock(&api_mode);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            db: Mutex::new(conn),
            api_key,
            api_mode,
            active_puuid: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::resolve_player,
            commands::get_match_history,
            commands::get_player_stats,
            commands::get_player_rank,
            commands::get_player_region,
            commands::get_challenger_standings,
            commands::get_grandmaster_standings,
            commands::get_master_standings,
            commands::get_platform_status,
            commands::get_active_game_status,
            commands::refresh_matches,
            commands::hud_bounds_enter,
            commands::hud_bounds_leave,
            commands::request_account_deletion,
        ])
        .setup(|app| {
            // Show dashboard on startup (overlay starts hidden)
            show_dashboard(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running HexForge Companion");
}

fn dirs_data_local() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok().map(std::path::PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_local_dir()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        dirs::data_local_dir()
    }
}