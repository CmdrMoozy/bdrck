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
use log::{self, Log, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError};
use std::io::{self, Write};

fn format_log_record(record: &LogRecord) -> String {
    format!(
        "[{} {}:{}] {} - {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        record.location().file(),
        record.location().line(),
        record.level(),
        record.args()
    )
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _: &LogMetadata) -> bool { true }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            writeln!(&mut io::stderr(), "{}", format_log_record(record)).unwrap();
        }
    }
}

pub fn try_init(
    max_log_level: Option<LogLevelFilter>,
) -> ::std::result::Result<(), SetLoggerError> {
    log::set_logger(|level_filter| {
        level_filter.set(max_log_level.unwrap_or(LogLevelFilter::Debug));
        Box::new(Logger)
    })
}

pub fn init(max_log_level: Option<LogLevelFilter>) { try_init(max_log_level).unwrap() }
