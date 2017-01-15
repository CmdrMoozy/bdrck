use chrono;
use log;
use std::io;

fn format_log_record(record: &log::LogRecord) -> String {
    format!("[{} {}:{}] {} - {}",
            chrono::offset::utc::UTC::now().format("%Y-%m-%d %H:%M:%S UTC"),
            record.location().file(),
            record.location().line(),
            record.level(),
            record.args())
}

pub struct DebugLogger;

impl log::Log for DebugLogger {
    fn enabled(&self, _: &log::LogMetadata) -> bool { true }

    fn log(&self, record: &log::LogRecord) {
        use std::io::Write;
        if self.enabled(record.metadata()) {
            writeln!(&mut io::stderr(), "{}", format_log_record(record)).unwrap();
        }
    }
}

/// Initialize a debug logger instance. This logger is useful for debugging
/// applications; it prints out very verbose messages, and includes extra
/// debugging information like timestamps.
pub fn init_debug_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Debug);
        Box::new(DebugLogger)
    })
}
