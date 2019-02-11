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

use crate::error::*;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::str::FromStr;

/// A Value is the value associated with a given flag. The type of the value
/// is different depending on the type of flag it is associated with.
#[derive(Debug, Eq, PartialEq)]
pub enum Value {
    /// A single string value (perhaps a string which the user of this library
    /// can then further interpret / parse as some other type).
    Single(String),
    /// A boolean value, from a flag defined as a boolean in its associated
    /// spec. It is represented here as a string, *but* Values internally
    /// enforces that all boolean strings must be successfully parseable.
    Boolean(String),
    /// A flag with repeated values. These should be treated the same as one
    /// would a Single string value, except there are potentially zero or more
    /// of them.
    Repeated(Vec<String>),
}

/// Values is a structure which contains all of the parsed command-line flag
/// values (or the default values for those flags). If parsing fails (including
/// if some required flags weren't specified, for example), an error is
/// returned.
///
/// This structure provides various accessor functions, to conveniently get at
/// the flag values. These accessors tend to follow the pattern of assuming the
/// caller is doing things correctly, and that the caller wants us to panic
/// early if something goes wrong. If this is not the desired behavior, the
/// Values::get accessor provides a safe API where the caller can do their own
/// error handling.
#[derive(Debug, Eq, PartialEq)]
pub struct Values {
    values: HashMap<String, Value>,
}

impl Values {
    /// Construct a new Values structure using the given default values, and
    /// the given values parsed from the program's command-line flags.
    pub fn new(default_values: HashMap<String, Value>, mut values: HashMap<String, Value>) -> Self {
        for (name, value) in default_values.into_iter() {
            values.entry(name).or_insert(value);
        }
        Values { values: values }
    }

    /// Returns whether or not there exists a `Value` for the given flag.
    pub fn contains_key(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Return the Value(s) of a single flag, as strings. The returned vector
    /// might be empty (if there is no Value associated with the flag), or it
    /// might contain exactly one entry (in the case of named or boolean flags),
    /// or it might contain many entries (in the case of positional flags).
    pub fn get(&self, name: &str) -> Vec<&str> {
        match self.values.get(name) {
            None => Vec::new(),
            Some(v) => match v {
                Value::Single(v) => vec![v.as_str()],
                Value::Boolean(v) => vec![v.as_str()],
                Value::Repeated(vs) => vs.iter().map(|v| v.as_str()).collect(),
            },
        }
    }

    /// Return the Value(s) of a single flag, parsed into the given type. This
    /// is a convenience wrapper around `get`.
    pub fn get_as<E, T: FromStr<Err = E>>(&self, name: &str) -> ::std::result::Result<Vec<T>, E> {
        self.get(name).iter().map(|v| v.parse::<T>()).collect()
    }
}

impl From<HashMap<String, Value>> for Values {
    fn from(values: HashMap<String, Value>) -> Self {
        Values { values: values }
    }
}

impl FromIterator<(String, Value)> for Values {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        let values: HashMap<String, Value> = iter.into_iter().collect();
        values.into()
    }
}

/// Take a single required flag value from a list of values. Generally this
/// should not be called directly, instead being used via #[command_callback].
pub fn take_required<T>(mut vs: Vec<T>) -> ValueResult<T> {
    match vs.len() {
        0 => Err(ValueError::CallbackParameter(format!(
            "expected one required value, found {}",
            vs.len()
        ))),
        1 => Ok(vs.remove(0)),
        _ => Err(ValueError::CallbackParameter(format!(
            "expected one required value, found {}",
            vs.len()
        ))),
    }
}
