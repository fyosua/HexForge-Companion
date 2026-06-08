use rusqlite::Connection;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

mod api;
mod db;
mod overlay;
mod commands;
mod process_watcher;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub api_key: String,
    pub api_mode: api::ApiMode,
    pub active_puuid: Mutex<Option<String>>,
    /// Shared flag that keeps the process-watcher thread alive.
    /// Set to `false` to stop the watcher.
    pub watcher_running: Arc<AtomicBool>,
}

/// Print a formatted startup banner with version, mode, and paths.
fn print_startup_banner(api_mode: &api::ApiMode, db_path: &std::path::Path) {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let separator = "\u{2550}".repeat(52);

    eprintln!("\n{}", separator);
    eprintln!("  {} v{}", name, version);
    eprintln!("  PID: {}", std::process::id());
    eprintln!("  DB:  {}", db_path.display());
    eprintln!("{}", separator);

    match api_mode {
        api::ApiMode::Mock => {
            eprintln!("  \u{1f527} MODE: Mock (offline) \u{2014} no API calls");
            eprintln!("  \u{2139}\u{fe0f}   Set RGAPI_KEY in .env for live data");
        }
        api::ApiMode::Direct { region, platform, .. } => {
            eprintln!("  \u{1f511} MODE: Direct \u{2014} live Riot API");
            eprintln!("     region:   {}", region);
            eprintln!("     platform: {}", platform);
        }
        api::ApiMode::Proxy { proxy_base } => {
            eprintln!("  \u{1f512} MODE: Proxy \u{2014} routed through backend");
            eprintln!("     proxy: {}", proxy_base);
        }
    }
    eprintln!("{}\n", separator);
}

/// Print a non-blocking warning if running in release (production) mode
/// without a live API key \u{2014} the app will only show mock data.
fn warn_if_production_mock(api_mode: &api::ApiMode) {
    if *api_mode == api::ApiMode::Mock && !cfg!(debug_assertions) {
        eprintln!(
            "[HexForge] \u{26a0}\u{fe0f}  RELEASE BUILD running in MOCK mode \u{2014} no RGAPI_KEY configured.\n\
             [HexForge]    Create a .env file with RGAPI_KEY=*** for live data, or\n\
             [HexForge]    set USE_MOCK=true in .env to silence this warning."
        );
    }
}

// \u{2014}\u{2014} System tray \u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}

/// Build the system tray icon and menu.
/// The tray provides show/hide/quit lifecycle control when the
/// overlay auto-hides on TFT detach.
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::TrayIconBuilder,
    };

    // Load the tray icon from bundled asset (ICO works cross-platform)
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.ico"))
        .expect("load tray icon");

    let show = MenuItem::with_id(app, "show", "Show Overlay", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Overlay", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit HexForge", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("HexForge Companion")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("overlay") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("overlay") {
                    let _ = window.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                tauri::tray::TrayIconEvent::DoubleClick {
                    button: tauri::tray::MouseButton::Left,
                    ..
                }
            ) {
                if let Some(app) = tray.app_handle() {
                    if let Some(window) = app.get_webview_window("overlay") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    eprintln!("[HexForge] Tray icon registered");
    Ok(())
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

    // \u{2014}\u{2014} Startup logging \u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}
    print_startup_banner(&api_mode, &db_path);
    warn_if_production_mock(&api_mode);

    // Shared flag for the process watcher thread.
    // The thread gets its own clone; dropping the AppState clone
    // after the app exits causes the watcher to stop automatically.
    let watcher_flag = Arc::new(AtomicBool::new(true));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Start TFT process watcher
            let handle = app.handle().clone();
            process_watcher::spawn_watcher(handle, 2000);
            eprintln!("[HexForge] Process watcher started (polling every 2s)");

            // Register updater plugin for auto-updates
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build()).ok();
            app.handle().plugin(tauri_plugin_process::init());

            // Register system tray (no tray on mobile)
            #[cfg(desktop)]
            if let Err(e) = setup_tray(app) {
                eprintln!("[HexForge] Tray setup failed: {e}");
            }

            Ok(())
        })
        .manage(AppState {
            db: Mutex::new(conn),
            api_key,
            api_mode,
            active_puuid: Mutex::new(None),
            watcher_running: watcher_flag,
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
