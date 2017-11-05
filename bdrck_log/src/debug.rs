// Copyright 2015 Axel Rasmussen
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono;
use log;
use std::io;

fn format_log_record(record: &log::LogRecord) -> String {
    format!("[{} {}:{}] {} - {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
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
