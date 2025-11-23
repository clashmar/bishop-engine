// engine_core/src/logging/logging.rs
use std::{fs::OpenOptions, sync::Mutex};
use log::LevelFilter;
use once_cell::sync::Lazy;
use simplelog::{Config, WriteLogger};
use crate::storage::editor_config::app_dir;

// TODO: Manage log file size
// TODO: More log types for onscreen_log

// Global mutable buffer that stores the most recent message.
pub static LAST_LOG: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

/// Helper macro that writes to the terminal and the screen.
#[macro_export]
macro_rules! onscreen_log {
    ($($arg:tt)*) => {{
        log::info!($($arg)*);
        let mut buf = $crate::logging::logging::LAST_LOG.lock().unwrap();
        *buf = format!($($arg)*);
    }};
}

pub fn init_file_logger() {
    let log_path = app_dir().join("bishop_engine.log");

    // Ensure the directory exists
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Open the file for appending
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(&log_path)
        .expect("Failed to open log file.");

    // Initialise the logger
    WriteLogger::init(LevelFilter::Info, Config::default(), file)
        .expect("Failed to initialise file logger");
}