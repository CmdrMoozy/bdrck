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

use log::{Level, LevelFilter, Log, Metadata, Record};
use logging::*;
use logging::write::*;
use regex::Regex;
use std::fmt::Arguments;

#[test]
fn test_parse_log_level_filter() {
    assert_eq!(LevelFilter::Off, parse_log_level_filter(" OfF ").unwrap());
    assert_eq!(
        LevelFilter::Error,
        parse_log_level_filter(" eRroR ").unwrap()
    );
    assert_eq!(LevelFilter::Warn, parse_log_level_filter(" wArN ").unwrap());
    assert_eq!(LevelFilter::Info, parse_log_level_filter(" InFo ").unwrap());
    assert_eq!(
        LevelFilter::Debug,
        parse_log_level_filter(" dEbUg ").unwrap()
    );
    assert_eq!(
        LevelFilter::Trace,
        parse_log_level_filter(" tRaCe ").unwrap()
    );

    assert!(parse_log_level_filter("foobar").is_err());
    assert!(parse_log_level_filter("").is_err());
    assert!(parse_log_level_filter("   ").is_err());
}

fn assert_log_filters_level(filters: &str, module_path: &str, expected_level: Option<LevelFilter>) {
    let filters: LogFilters = filters.parse().unwrap();
    assert_eq!(expected_level, filters.max_level_for(module_path));
}

#[test]
fn test_log_filters() {
    assert_log_filters_level("info", "main", Some(LevelFilter::Info));
    assert_log_filters_level(
        "main=info;foo::bar=debug",
        "main::submodule",
        Some(LevelFilter::Info),
    );
    assert_log_filters_level("main=info;foo::bar=debug", "bar", None);
    assert_log_filters_level("main=info;foo::bar=debug", "foo", None);
    assert_log_filters_level(
        "main=info;foo::bar=debug",
        "foo::bar",
        Some(LevelFilter::Debug),
    );
    assert_log_filters_level(
        "main=info;foo::bar=debug",
        "foo::bar::baz",
        Some(LevelFilter::Debug),
    );
}

fn test_metadata(level: Level) -> Metadata<'static> {
    Metadata::builder().level(level).build()
}

fn test_record<'a>(args: Arguments<'a>, level: Level) -> Record<'a> {
    Record::builder()
        .args(args)
        .metadata(test_metadata(level))
        .level(level)
        .target("target")
        .module_path(Some("bdrck::tests::logging"))
        .file(Some("logging.rs"))
        .line(Some(1234))
        .build()
}

// This function normalizes the output from the Logging implementation,
// replacing things which are unpredictable in unit tests like timestamps.
fn normalize_log_output(output: &str) -> String {
    lazy_static! {
        static ref DATE_REGEX: Regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} UTC").unwrap();
    }
    DATE_REGEX
        .replace_all(output, "2018-01-01 12:34:56 UTC")
        .into_owned()
}

#[test]
fn test_logger_enabled() {
    let logger = Logger::new(Some("error".parse().unwrap()), None, false).unwrap();
    assert!(logger.enabled(&test_metadata(Level::Error)));
    assert!(!logger.enabled(&test_metadata(Level::Warn)));

    let logger = Logger::new(Some("info".parse().unwrap()), None, false).unwrap();
    assert!(logger.enabled(&test_metadata(Level::Warn)));
    assert!(logger.enabled(&test_metadata(Level::Info)));
    assert!(!logger.enabled(&test_metadata(Level::Debug)));
}

#[test]
fn test_logging_output() {
    let log_output_buffer: Vec<u8> = Vec::new();
    let adapter = SyncWriteAdapter::new(log_output_buffer);
    let logger = Logger::new(
        Some("info".parse().unwrap()),
        Some(new_log_output_factory(adapter.clone())),
        true,
    ).unwrap();

    logger.log(&test_record(format_args!("foo"), Level::Error));
    logger.log(&test_record(format_args!("bar"), Level::Warn));
    logger.log(&test_record(format_args!("baz"), Level::Info));
    logger.log(&test_record(format_args!("quux"), Level::Debug));
    logger.log(&test_record(format_args!("oof"), Level::Trace));
    logger.flush();

    let log_output = normalize_log_output(&String::from_utf8(adapter.lock().clone()).unwrap());
    assert_eq!(
        [
            "[2018-01-01 12:34:56 UTC logging.rs:1234] ERROR - foo",
            "[2018-01-01 12:34:56 UTC logging.rs:1234] WARN - bar",
            "[2018-01-01 12:34:56 UTC logging.rs:1234] INFO - baz\n",
        ].join("\n"),
        log_output
    );
}
