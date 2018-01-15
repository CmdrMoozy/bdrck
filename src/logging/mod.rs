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
use log::{set_boxed_logger, Level, LevelFilter, Log, Metadata, Record};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

const RUST_LOG_ENV_VAR: &'static str = "RUST_LOG";

pub fn parse_log_level_filter(s: &str) -> Result<LevelFilter> {
    lazy_static! {
        static ref STRING_MAPPING: HashMap<String, LevelFilter> = {
            let mut m = HashMap::new();
            m.insert(LevelFilter::Off.to_string().to_lowercase(), LevelFilter::Off);
            m.insert(LevelFilter::Error.to_string().to_lowercase(), LevelFilter::Error);
            m.insert(LevelFilter::Warn.to_string().to_lowercase(), LevelFilter::Warn);
            m.insert(LevelFilter::Info.to_string().to_lowercase(), LevelFilter::Info);
            m.insert(LevelFilter::Debug.to_string().to_lowercase(), LevelFilter::Debug);
            m.insert(LevelFilter::Trace.to_string().to_lowercase(), LevelFilter::Trace);
            m
        };
    }

    let normalized = s.trim().to_lowercase();
    match STRING_MAPPING.get(&normalized) {
        None => bail!("Invalid LevelFilter '{}'", s),
        Some(f) => Ok(*f),
    }
}

pub struct LogFilter {
    pub module: Option<Regex>,
    pub level: LevelFilter,
}

impl LogFilter {
    pub fn max_level_for(&self, module_path: &str) -> Option<LevelFilter> {
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
    pub fn max_level_for(&self, module_path: &str) -> Option<LevelFilter> {
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

fn format_log_record(record: &Record) -> String {
    format!(
        "[{} {}:{}] {} - {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        record.file().unwrap_or("UNKNOWN_FILE"),
        record
            .line()
            .map_or("UNKNOWN_LINE".to_owned(), |l| l.to_string()),
        record.level(),
        record.args()
    )
}

pub struct Logger {
    filters: Option<LogFilters>,
    max_level: Option<LevelFilter>,
}

impl Logger {
    pub fn new(mut filters: Option<LogFilters>) -> Result<Logger> {
        if filters.is_none() {
            let filters_str: Option<String> = get_env_var(RUST_LOG_ENV_VAR)?;
            if let Some(filters_str) = filters_str {
                filters = Some(filters_str.parse()?);
            }
        }

        let max_level: Option<LevelFilter> = match filters {
            None => None,
            Some(ref fs) => fs.0.iter().map(|f| f.level).max(),
        };

        Ok(Logger {
            filters: filters,
            max_level: max_level,
        })
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let max_level: Option<Level> = self.max_level.map(|lf| lf.to_level()).unwrap_or(None);
        match max_level {
            None => true,
            Some(level) => metadata.level() <= level,
        }
    }

    fn log(&self, record: &Record) {
        let module_path: &str = record.module_path().unwrap_or("UNKNOWN_MODULE");
        let max_level_filter: Option<LevelFilter> = self.filters
            .as_ref()
            .map(|fs| fs.max_level_for(module_path))
            .unwrap_or(None);
        let max_level: Option<Level> = max_level_filter.map(|mlf| mlf.to_level()).unwrap_or(None);

        let enabled = match max_level {
            None => true,
            Some(level) => record.level() <= level,
        };
        if enabled {
            eprintln!("{}", format_log_record(record));
        }
    }

    fn flush(&self) {}
}

pub fn try_init(filters: Option<LogFilters>) -> Result<()> {
    Ok(set_boxed_logger(Box::new(Logger::new(filters)?))?)
}

pub fn init(filters: Option<LogFilters>) { try_init(filters).unwrap() }
