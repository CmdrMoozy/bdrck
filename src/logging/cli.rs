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

use logging::debug::DebugLogger;
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
            if record.level() == log::LogLevel::Info {
                writeln!(&mut io::stdout(), "{}", format_log_record(record)).unwrap();
            } else if record.level() == log::LogLevel::Error ||
                      record.level() == log::LogLevel::Warn {
                writeln!(&mut io::stderr(), "{}", format_log_record(record)).unwrap();
            } else {
                self.debug_logger.log(record);
            }
        }
    }
}

/// Initialize a command-line-interface logger instance. This logger is
/// intended to be used by command-line programs which use info!/warn!/error!
/// in order to display information to the user (instead of just, e.g.,
/// println!).
pub fn init_cli_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Debug);
        Box::new(CliLogger { debug_logger: DebugLogger {} })
    })
}
