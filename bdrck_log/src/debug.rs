use log;
use std::boxed::Box;
use std::io;

use super::format;

struct DebugLogger;

impl log::Log for DebugLogger {
    fn enabled(&self, _: &log::LogMetadata) -> bool { true }

    fn log(&self, record: &log::LogRecord) {
        use std::io::Write;
        if self.enabled(record.metadata()) {
            writeln!(&mut io::stderr(), "{}", format::format_log_record(record)).unwrap();
        }
    }
}

pub fn init_debug_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Debug);
        Box::new(DebugLogger)
    })
}