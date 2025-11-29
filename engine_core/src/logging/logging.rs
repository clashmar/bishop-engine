// engine_core/src/logging/logging.rs
use std::io::*;
use std::sync::Mutex;
use flexi_logger::*;
use log::Record;
use once_cell::sync::Lazy;
use crate::storage::editor_config::app_dir;

// Global mutable buffer that stores the most recent message.
pub static LAST_LOG: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

/// Helper macro that allow logs to be displayed by 
/// the program and printed to the console.
#[macro_export]
macro_rules! onscreen_log {
    ($lvl:expr, $($arg:tt)*) => {{
        println!($($arg)*);
        log::log!($lvl, $($arg)*);
        let mut buf = $crate::logging::logging::LAST_LOG.lock().unwrap();
        *buf = format!($($arg)*);
    }};
}

/// Helper macro that allow logs to be displayed by the program.
#[macro_export]
macro_rules! onscreen_info  { ($($arg:tt)*) => { onscreen_log!(log::Level::Info,  $($arg)*) }; }
/// Helper macro that allow logs to be displayed by the program.

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