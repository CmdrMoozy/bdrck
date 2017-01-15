use ::debug::DebugLogger;
use log;
use std::io;

fn format_log_record(record: &log::LogRecord) -> String {
    if record.level() == log::LogLevel::Info {
        format!("{}", record.args())
    } else {
        format!("{}: {}", record.level(), record.args())
    }
}

pub struct CliLogger {
    debug_logger: DebugLogger,
}

impl log::Log for CliLogger {
    fn enabled(&self, _: &log::LogMetadata) -> bool { true }

    fn log(&self, record: &log::LogRecord) {
        use std::io::Write;
        if self.enabled(record.metadata()) {
            if record.level() == log::LogLevel::Error || record.level() == log::LogLevel::Warn ||
               record.level() == log::LogLevel::Info {
                writeln!(&mut io::stderr(), "{}", format_log_record(record)).unwrap();
            } else {
                self.debug_logger.log(record);
            }
        }
    }
}

pub fn init_cli_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Debug);
        Box::new(CliLogger { debug_logger: DebugLogger {} })
    })
}
