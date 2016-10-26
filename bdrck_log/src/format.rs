use chrono;
use log;
use std::string::String;

pub fn format_log_record(record: &log::LogRecord) -> String {
    format!("[{} {}:{}] {} - {}",
            chrono::offset::utc::UTC::now().format("%Y-%m-%d %H:%M:%S UTC"),
            record.location().file(),
            record.location().line(),
            record.level(),
            record.args())
}
