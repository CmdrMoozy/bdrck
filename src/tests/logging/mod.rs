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

use log::{Level, LevelFilter, Log, Metadata};
use logging::*;

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

#[test]
fn test_logger_enabled() {
    let logger = Logger::new(Some("error".parse().unwrap())).unwrap();
    assert!(logger.enabled(&Metadata::builder().level(Level::Error).build()));
    assert!(!logger.enabled(&Metadata::builder().level(Level::Warn).build()));

    let logger = Logger::new(Some("info".parse().unwrap())).unwrap();
    assert!(logger.enabled(&Metadata::builder().level(Level::Warn).build()));
    assert!(logger.enabled(&Metadata::builder().level(Level::Info).build()));
    assert!(!logger.enabled(&Metadata::builder().level(Level::Debug).build()));
}
