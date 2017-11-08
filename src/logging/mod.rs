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
use error::*;
use log::{self, Log, LogLevel, LogLevelFilter, LogMetadata, LogRecord};
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, Write};
use std::str::FromStr;

const RUST_LOG_ENV_VAR: &'static str = "RUST_LOG";

pub fn parse_log_level_filter(s: &str) -> Result<LogLevelFilter> {
    lazy_static! {
        static ref STRING_MAPPING: HashMap<String, LogLevelFilter> = {
            let mut m = HashMap::new();
            m.insert(LogLevelFilter::Off.to_string().to_lowercase(), LogLevelFilter::Off);
            m.insert(LogLevelFilter::Error.to_string().to_lowercase(), LogLevelFilter::Error);
            m.insert(LogLevelFilter::Warn.to_string().to_lowercase(), LogLevelFilter::Warn);
            m.insert(LogLevelFilter::Info.to_string().to_lowercase(), LogLevelFilter::Info);
            m.insert(LogLevelFilter::Debug.to_string().to_lowercase(), LogLevelFilter::Debug);
            m.insert(LogLevelFilter::Trace.to_string().to_lowercase(), LogLevelFilter::Trace);
            m
        };
    }

    let normalized = s.trim().to_lowercase();
    match STRING_MAPPING.get(&normalized) {
        None => bail!("Invalid LogLevelFilter '{}'", s),
        Some(f) => Ok(*f),
    }
}

pub struct LogFilter {
    pub module: Option<Regex>,
    pub level: LogLevelFilter,
}

impl LogFilter {
    pub fn max_level_for(&self, module_path: &str) -> Option<LogLevelFilter> {
        match self.module {
            None => Some(self.level),
            Some(ref module) => match module.is_match(module_path) {
                false => None,
                true => Some(self.level),
            },
        }
    }
}

impl FromStr for LogFilter {
    type Err = Error;

    fn from_str(s: &str) -> Result<LogFilter> {
        match s.rfind('=') {
            None => Ok(LogFilter {
                module: None,
                level: parse_log_level_filter(s)?,
            }),
            Some(eq_pos) => {
                let mut re: String = "^".to_owned();
                re.push_str(&s[..eq_pos]);
                Ok(LogFilter {
                    module: Some(Regex::new(&re)?),
                    level: parse_log_level_filter(&s[eq_pos + 1..])?,
                })
            },
        }
    }
}

pub struct LogFilters(pub Vec<LogFilter>);

impl LogFilters {
    pub fn max_level_for(&self, module_path: &str) -> Option<LogLevelFilter> {
        self.0
            .iter()
            .map(|f| f.max_level_for(module_path))
            .filter(|l| l.is_some())
            .next()
            .unwrap_or(None)
    }
}

impl FromStr for LogFilters {
    type Err = Error;

    /// Parse a set of log filters from a string.
    ///
    /// We assume that the regex contained in log filters will only contain
    /// certain characters: those which can appear in valid module names
    /// (something like [A-Za-z_][A-Za-z0-9_]* separated by :'s), and maybe
    /// some modifiers or etc., like *+?|(){}[].
    ///
    /// But, we want a string to contain *several* filters. So, using the
    /// above assumption about what characters will appear in the regex,
    /// we'll use the ; character as a separator. So, the final format is:
    ///
    /// regex=level;regex=level;...
    fn from_str(s: &str) -> Result<LogFilters> {
        let filters: Result<Vec<LogFilter>> = s.split(';').map(|f| f.parse()).collect();
        Ok(LogFilters(filters?))
    }
}

fn get_env_var(key: &str) -> Result<Option<String>> {
    match ::std::env::var(key) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            ::std::env::VarError::NotPresent => Ok(None),
            ::std::env::VarError::NotUnicode(_) => {
                bail!("Environment variable '{}' not valid unicode", key)
            },
        },
    }
}

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

struct Logger {
    filters: Option<LogFilters>,
    max_level: Option<LogLevelFilter>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        let max_level: Option<LogLevel> =
            self.max_level.map(|lf| lf.to_log_level()).unwrap_or(None);
        match max_level {
            None => true,
            Some(level) => metadata.level() <= level,
        }
    }

    fn log(&self, record: &LogRecord) {
        let module_path: &str = record.location().module_path();
        let max_level_filter: Option<LogLevelFilter> = self.filters
            .as_ref()
            .map(|fs| fs.max_level_for(module_path))
            .unwrap_or(None);
        let max_level: Option<LogLevel> = max_level_filter
            .map(|mlf| mlf.to_log_level())
            .unwrap_or(None);

        let enabled = match max_level {
            None => true,
            Some(level) => record.level() <= level,
        };
        if enabled {
            writeln!(&mut io::stderr(), "{}", format_log_record(record)).unwrap();
        }
    }
}

pub fn try_init(mut filters: Option<LogFilters>) -> Result<()> {
    if filters.is_none() {
        let filters_str: Option<String> = get_env_var(RUST_LOG_ENV_VAR)?;
        if let Some(filters_str) = filters_str {
            filters = Some(filters_str.parse()?);
        }
    }

    let max_level: Option<LogLevelFilter> = match filters {
        None => None,
        Some(ref fs) => fs.0.iter().map(|f| f.level).max(),
    };

    Ok(log::set_logger(|level_filter| {
        if let Some(level) = max_level {
            level_filter.set(level);
        }
        Box::new(Logger {
            filters: filters,
            max_level: max_level,
        })
    })?)
}

pub fn init(filters: Option<LogFilters>) { try_init(filters).unwrap() }
