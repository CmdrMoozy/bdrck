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

use log::LogLevelFilter;
use logging::*;

#[test]
fn test_parse_log_level_filter() {
    assert_eq!(
        LogLevelFilter::Off,
        parse_log_level_filter(" OfF ").unwrap()
    );
    assert_eq!(
        LogLevelFilter::Error,
        parse_log_level_filter(" eRroR ").unwrap()
    );
    assert_eq!(
        LogLevelFilter::Warn,
        parse_log_level_filter(" wArN ").unwrap()
    );
    assert_eq!(
        LogLevelFilter::Info,
        parse_log_level_filter(" InFo ").unwrap()
    );
    assert_eq!(
        LogLevelFilter::Debug,
        parse_log_level_filter(" dEbUg ").unwrap()
    );
    assert_eq!(
        LogLevelFilter::Trace,
        parse_log_level_filter(" tRaCe ").unwrap()
    );

    assert!(parse_log_level_filter("foobar").is_err());
    assert!(parse_log_level_filter("").is_err());
    assert!(parse_log_level_filter("   ").is_err());
}

fn assert_log_filters_level(
    filters: &str,
    module_path: &str,
    expected_level: Option<LogLevelFilter>,
) {
    let filters: LogFilters = filters.parse().unwrap();
    assert_eq!(expected_level, filters.max_level_for(module_path));
}

#[test]
fn test_log_filters() {
    assert_log_filters_level("info", "main", Some(LogLevelFilter::Info));
    assert_log_filters_level(
        "main=info;foo::bar=debug",
        "main::submodule",
        Some(LogLevelFilter::Info),
    );
    assert_log_filters_level("main=info;foo::bar=debug", "bar", None);
    assert_log_filters_level("main=info;foo::bar=debug", "foo", None);
    assert_log_filters_level(
        "main=info;foo::bar=debug",
        "foo::bar",
        Some(LogLevelFilter::Debug),
    );
    assert_log_filters_level(
        "main=info;foo::bar=debug",
        "foo::bar::baz",
        Some(LogLevelFilter::Debug),
    );
}
