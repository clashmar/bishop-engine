// engine_core/src/logging/mod.rs
use crate::storage::editor_config::app_dir;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use flexi_logger::*;
use log::Record;
use std::io::*;

pub use crate::onscreen_log;
pub use crate::onscreen_debug;
pub use crate::onscreen_info;
pub use crate::onscreen_warn;
pub use crate::onscreen_error;

const MAX_LOG_ENTRIES: usize = 500;

#[derive(Clone)]
pub struct LogEntry {
    pub level: log::Level,
    pub message: String,
}

/// Stores log entries with a monotonically increasing counter for change detection.
#[derive(Default)]
pub struct LogHistory {
    entries: Vec<LogEntry>,
    total_pushed: usize,
}

impl LogHistory {
    pub fn push(&mut self, level: log::Level, message: String) {
        if self.entries.len() >= MAX_LOG_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(LogEntry { level, message });
        self.total_pushed += 1;
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn last(&self) -> Option<&LogEntry> {
        self.entries.last()
    }

    /// Returns the total number of entries ever pushed, even after cycling.
    pub fn total_pushed(&self) -> usize {
        self.total_pushed
    }

    /// Clears all entries but preserves total_pushed for change detection.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub static LOG_HISTORY: Lazy<Mutex<LogHistory>> = Lazy::new(|| Mutex::new(LogHistory::default()));

// Global mutable buffer that stores the most recent message (kept for backwards compatibility).
pub static LAST_LOG: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

/// Helper macro that allow logs to be displayed by
/// the program and printed to the console.
#[macro_export]
macro_rules! onscreen_log {
    ($lvl:expr, $($arg:tt)*) => {{
        println!($($arg)*);
        log::log!($lvl, $($arg)*);
        let msg = format!($($arg)*);
        {
            let mut history = $crate::logging::LOG_HISTORY.lock().unwrap();
            history.push($lvl, msg.clone());
        }
        let mut buf = $crate::logging::LAST_LOG.lock().unwrap();
        *buf = msg;
    }};
}

/// Helper macro that allow logs to be displayed by the program.
#[macro_export]
macro_rules! onscreen_info  { ($($arg:tt)*) => { onscreen_log!(log::Level::Info,  $($arg)*) }; }

/// Helper macro that allow logs to be displayed by the program.
#[macro_export]
macro_rules! onscreen_warn  { ($($arg:tt)*) => { onscreen_log!(log::Level::Warn,  $($arg)*) }; }

/// Helper macro that allow logs to be displayed by the program.
#[macro_export]
macro_rules! onscreen_error { ($($arg:tt)*) => { onscreen_log!(log::Level::Error, $($arg)*) }; }

/// Helper macro that allow logs to be displayed by the program.
#[macro_export]
macro_rules! onscreen_debug { ($($arg:tt)*) => { onscreen_log!(log::Level::Debug, $($arg)*) }; }

/// Initializes the system logger.
pub fn init_file_logger() {
    let log_dir = app_dir().join("logs");

    let file_spec = FileSpec::default()
        .directory(&log_dir)         
        .basename("bishop_engine")
        .suffix("log");

    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(file_spec)
        .format(my_formatter)
        .rotate(
            Criterion::Size(5_000_000),
            Naming::Numbers,
            Cleanup::KeepLogFiles(5),
        )
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .expect("Unable to init logger.");

    onscreen_info!("Log dir: {}.", &log_dir.display());

    fn my_formatter(
        write: &mut dyn Write, 
        now: &mut DeferredNow, 
        record: &Record
    ) -> Result<()> {
        write!(
            write,
            "{} {:5} [{}] {}",
            now.format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.module_path().unwrap_or("<unknown>"),
            &record.args()
        )
    }
}