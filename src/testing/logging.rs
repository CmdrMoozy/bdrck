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

use log::{set_logger, set_max_level, LevelFilter, Log, Metadata, Record};

static TEST_LOGGER: TestLogger = TestLogger;

struct TestLogger;

impl Log for TestLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("{} {} {}", record.level(), record.target(), record.args());
    }

    fn flush(&self) {}
}

/// Call set_logger with a test-friendly logger. Logging will be enabled at the
/// given level, or at "Debug" if no other level was specified.
pub fn set_test_logger(max_log_level: Option<LevelFilter>) {
    set_max_level(max_log_level.unwrap_or(LevelFilter::Debug));
    set_logger(&TEST_LOGGER).unwrap();
}
