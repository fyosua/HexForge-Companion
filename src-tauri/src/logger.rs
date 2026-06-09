use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Initialize the log file. Creates the logs directory under the app data folder.
/// Call once at startup from `run()`.
pub fn init_logger(app_data_dir: &PathBuf) {
    let log_dir = app_data_dir.join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let mut guard = LOG_DIR.lock().unwrap();
    *guard = Some(log_dir);
}

/// Write a log line to the log file + stderr.
/// Timestamped automatically.
pub fn log_line(line: &str) {
    // Always write to stderr (visible in terminal)
    eprintln!("{}", line);

    // Also write to log file if initialized
    let guard = LOG_DIR.lock().unwrap();
    if let Some(dir) = guard.as_ref() {
        let path = dir.join("app.log");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{}", line);
        }
    }
}

/// Macro that logs with a [HexForge] prefix and timestamp.
#[macro_export]
macro_rules! hlog {
    ($($arg:tt)*) => {
        $crate::logger::log_line(&format!("[HexForge] {}", format_args!($($arg)*)))
    };
}