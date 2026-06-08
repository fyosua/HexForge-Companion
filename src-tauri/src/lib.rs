use rusqlite::Connection;
use std::sync::Mutex;

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

    // Log startup mode
    match &api_mode {
        api::ApiMode::Mock => eprintln!("[HexForge] 🔧 Mock mode — no API calls will be made"),
        api::ApiMode::Direct { region, platform, .. } => {
            eprintln!("[HexForge] 🔑 Direct API mode — region={region}, platform={platform}");
        }
        api::ApiMode::Proxy { proxy_base } => {
            eprintln!("[HexForge] 🔒 Proxy API mode — proxy={proxy_base}");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            commands::hud_bounds_enter,
            commands::hud_bounds_leave,
            commands::request_account_deletion,
        ])
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