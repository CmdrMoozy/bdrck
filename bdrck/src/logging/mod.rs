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

/// write provides adapters to use anything which implements Write as a logging
/// destination.
pub mod write;

use crate::error::*;
use crate::logging::write::*;
use chrono;
use lazy_static::lazy_static;
use log::{self, LevelFilter, Log, Metadata, Record};
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

const RUST_LOG_ENV_VAR: &'static str = "RUST_LOG";

/// This is a utility function which provides a way to parse a log::LevelFilter
/// from a string. Upstream doesn't impl FromStr for this type, so this extra
/// utility is necessary.
pub fn parse_log_level_filter(s: &str) -> Result<LevelFilter> {
    lazy_static! {
        static ref STRING_MAPPING: HashMap<String, LevelFilter> = {
            let mut m = HashMap::new();
            m.insert(
                LevelFilter::Off.to_string().to_lowercase(),
                LevelFilter::Off,
            );
            m.insert(
                LevelFilter::Error.to_string().to_lowercase(),
                LevelFilter::Error,
            );
            m.insert(
                LevelFilter::Warn.to_string().to_lowercase(),
                LevelFilter::Warn,
            );
            m.insert(
                LevelFilter::Info.to_string().to_lowercase(),
                LevelFilter::Info,
            );
            m.insert(
                LevelFilter::Debug.to_string().to_lowercase(),
                LevelFilter::Debug,
            );
            m.insert(
                LevelFilter::Trace.to_string().to_lowercase(),
                LevelFilter::Trace,
            );
            m
        };
    }

    let normalized = s.trim().to_lowercase();
    match STRING_MAPPING.get(&normalized) {
        None => {
            return Err(Error::InvalidArgument(format!(
                "invalid LevelFilter '{}'",
                s
            )));
        }
        Some(f) => Ok(*f),
    }
}

/// A LogFilter is a single filter, perhaps one of many, that can be applied to
/// log messages before actually outputting them.
pub struct LogFilter {
    /// This LogFilter is intended to be applied to any modules which match this
    /// regular expression. If this field is None instead, then this LogFilter
    /// should be applied to *all* modules.
    pub module: Option<Regex>,

    /// The LevelFilter which should be applied to matching modules.
    pub level: LevelFilter,
}

impl LogFilter {
    /// The LevelFilter this LogFilter applies to the given module. If this
    /// LogFilter does not match the given module, then None is returned
    /// instead.
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
            }
        }
    }
}

/// LogFilters is a structure which defines the full set of filters a Logger
/// should apply to log messages before actually outputting them.
pub struct LogFilters(pub Vec<LogFilter>);

impl LogFilters {
    /// Returns the LevelFilter which should be applied to the given module. If
    /// no LogFilter entries apply to the given module, Trace is returned
    /// instead, since logging messages are enabled by default in this library.
    /// If multiple LevelFilters matched the given module, then the *lowest*
    /// (i.e., most restrictive) LevelFilter is returned.
    pub fn max_level_for(&self, module_path: &str) -> LevelFilter {
        self.0
            .iter()
            .filter_map(|f| f.max_level_for(module_path))
            .min()
            .unwrap_or(LevelFilter::Trace)
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

/// Options is a structure which describes the options for a Logger. Generally
/// these should be constructed using OptionsBuilder, instead of filling in all
/// fields by hand.
pub struct Options {
    /// Filters controlling which log statements are enabled. If unspecified,
    /// defaults to the value of the RUST_LOG environment variable. If that is
    /// also unspecified, then by default all logging statements are enabled.
    pub filters: LogFilters,
    /// The global maximum enabled logging level. This is basically the highest
    /// level configured in any of `filters`, or `LevelFilter::Trace` by default
    /// (meaning that all log messages are enabled, if no filters are
    /// specified).
    pub max_level: LevelFilter,
    /// Where to write log output to. If unspecified, defaults to stderr.
    pub output_factory: LogOutputFactory,
    /// Whether or not a log output (or flush) failure should result in a panic.
    /// Panicing is the "safest" option in some sense, but the default behavior
    /// is to silently ignore failures (under the assumption that most of the
    /// time users will want the application to continue working even if it
    /// can't produce log output).
    pub panic_on_output_failure: bool,
    /// If true, *always* call flush() after each log statement. This has a
    /// significant negative impact on performance, but it does mean that log
    /// statements will appear immediately, which may be useful e.g. for
    /// debugging. By default, this feature is disabled.
    pub always_flush: bool,
}

/// OptionsBuilder provides a builder-style interface to construct an Options.
pub struct OptionsBuilder {
    filters: Option<LogFilters>,
    output_factory: Option<LogOutputFactory>,
    panic_on_output_failure: Option<bool>,
    always_flush: Option<bool>,
}

impl OptionsBuilder {
    /// Construct a new OptionsBuilder, which by default just sets the options
    /// to their default values.
    pub fn new() -> Self {
        OptionsBuilder {
            filters: None,
            output_factory: None,
            panic_on_output_failure: None,
            always_flush: None,
        }
    }

    /// Set the filters which will be applied to any logging calls before
    /// actually logging them.
    pub fn set_filters(mut self, filters: LogFilters) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set the output factory the Logger should use. This can be used to
    /// redirect logging output more generically, although for the common case
    /// set_output_to might be easier to use.
    pub fn set_output_factory(mut self, output_factory: LogOutputFactory) -> Self {
        self.output_factory = Some(output_factory);
        self
    }

    /// Configure the Logger to write its log output to the given Write
    /// implementation (e.g. a File). This is a convenience function which
    /// just calls set_output_factory under the hood.
    pub fn set_output_to<T: Write + Send + 'static>(self, output_writer: T) -> Self {
        self.set_output_factory(new_log_output_factory(output_writer))
    }

    /// Set whether or not the Logger should panic! if writing log output fails.
    /// Generally this can be useful for debugging, but in real production code
    /// losing a log entry might not be a good enough reason to terminate the
    /// entire process.
    pub fn set_panic_on_output_failure(mut self, panic_on_output_failure: bool) -> Self {
        self.panic_on_output_failure = Some(panic_on_output_failure);
        self
    }

    /// Set whether or not the Logger should flush its output after every
    /// logging call. The benefit is that log messages show up immediately, and
    /// are less susceptible to "corruption" when logging from multiple threads,
    /// but the cost is a significant performance penalty for all logging calls.
    pub fn set_always_flush(mut self, always_flush: bool) -> Self {
        self.always_flush = Some(always_flush);
        self
    }

    /// Build an Options structure from this builder's current state. This might
    /// return an error if no custom filters were specified, and attempting to
    /// retrieve filters from an environment variable fails.
    pub fn build(self) -> Result<Options> {
        let filters: LogFilters = match self.filters {
            None => match get_env_var(RUST_LOG_ENV_VAR)? {
                None => LogFilters(vec![]),
                Some(filters_str) => filters_str.parse()?,
            },
            Some(filters) => filters,
        };
        let max_level: LevelFilter = filters
            .0
            .iter()
            .map(|f| f.level)
            .max()
            .unwrap_or(LevelFilter::Trace);

        Ok(Options {
            filters: filters,
            max_level: max_level,
            output_factory: self
                .output_factory
                .unwrap_or_else(|| Box::new(|| Box::new(::std::io::stderr()))),
            panic_on_output_failure: self.panic_on_output_failure.unwrap_or(false),
            always_flush: self.always_flush.unwrap_or(false),
        })
    }
}

fn get_env_var(key: &str) -> Result<Option<String>> {
    match ::std::env::var(key) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            ::std::env::VarError::NotPresent => Ok(None),
            _ => Err(Error::EnvVar(e)),
        },
    }
}

/// This function formats the given log Record into a string, which can then be
/// written directly to the logging sink (e.g. stderr, a file, etc.).
pub fn format_log_record(record: &Record) -> String {
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

/// This is the standard Log implementation bdrck provides, which can be
/// controlled by setting its Options. In general, this is intended to be a
/// pretty versatile implementation, suitable for both command-line applications
/// as well as serving daemons, given the right Options.
pub struct Logger {
    options: Options,
}

impl Logger {
    /// Construct a new Logger with the given Options controlling its behavior.
    pub fn new(options: Options) -> Self {
        Logger { options: options }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.options.max_level
    }

    fn log(&self, record: &Record) {
        if record.level()
            > self
                .options
                .filters
                .max_level_for(record.module_path().unwrap_or(""))
        {
            return;
        }

        let res = write!(
            (self.options.output_factory)(),
            "{}\n",
            format_log_record(record)
        );
        if self.options.panic_on_output_failure {
            if let Err(e) = res {
                panic!("Failed to write log output: {}", e);
            } else {
                return;
            }
        }
        if self.options.always_flush {
            self.flush();
        }
    }

    fn flush(&self) {
        let res = (self.options.output_factory)().flush();
        if self.options.panic_on_output_failure {
            if let Err(e) = res {
                panic!("Failed to flush log output: {}", e);
            }
        }
    }
}

/// Try to set up a new global Logger, with the given Options controlling its
/// behavior, returning a standard error if doing so fails in some way.
pub fn try_init(options: Options) -> Result<()> {
    let logger = Logger::new(options);
    log::set_max_level(logger.options.max_level);
    log::set_boxed_logger(Box::new(logger))?;
    Ok(())
}

/// This is a shortcut which just calls try_init(), but implicitly unwraps the
/// resulting error. For applications which initialize a Logger first thing in
/// main(), this is probably a reasonable choice.
pub fn init(options: Options) {
    try_init(options).unwrap();
}
