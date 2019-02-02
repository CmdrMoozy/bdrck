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
    /// pec.
    Boolean(bool),
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

    /// Looks up a generic flag value, which may or may not be present. This
    /// function is guaranteed not to panic, but error handling and type
    /// matching is left up to the caller do deal with at runtime.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    /// Lookup a single optional named flag value. This function panics if the
    /// flag has a value, but it is of the wrong type.
    pub fn get_single(&self, name: &str) -> Option<&str> {
        match self.values.get(name) {
            None => None,
            Some(v) => match v {
                &Value::Single(ref s) => Some(s.as_str()),
                _ => panic!("Flag '{}' is not a named non-boolean flag", name),
            },
        }
    }

    /// Lookup a required named flag value. This function panics if the value is
    /// not found, or if the flag with the given name is of the wrong type.
    pub fn get_required(&self, name: &str) -> &str {
        match self.values.get(name) {
            None => panic!("Missing required flag value for '{}'", name),
            Some(v) => match v {
                &Value::Single(ref s) => s.as_str(),
                _ => panic!("Flag '{}' is not a named non-boolean flag", name),
            },
        }
    }

    /// Lookup a required named flag value, moving the value into a new
    /// structure of the given type. A convenience wrapper around
    /// get_required.
    pub fn get_required_as<T: From<String>>(&self, name: &str) -> T {
        T::from(self.get_required(name).to_owned())
    }

    /// Lookup a required named flag value, parsing the string into the given
    /// type. A convenience wrapper around get_required.
    pub fn get_required_parsed<E, T: FromStr<Err = E>>(
        &self,
        name: &str,
    ) -> ::std::result::Result<T, E> {
        self.get_required(name).parse()
    }

    /// Lookup a boolean flag value. Boolean flags always have a value, since
    /// they have an implicit default value of false. This function panics if
    /// the value is not found, or if the flag with the given name is of the
    /// wrong type.
    pub fn get_boolean(&self, name: &str) -> bool {
        match self.values.get(name) {
            None => panic!("Missing boolean flag value for '{}'", name),
            Some(v) => match v {
                &Value::Boolean(b) => b,
                _ => panic!("Flag '{}' is not a named boolean flag", name),
            },
        }
    }

    /// This function looks up a positional flag's values, returning the full
    /// (possibly empty) list of values. This function panics if no associated
    /// value list was found, or if the flag with the given name is of the wrong
    /// type.
    pub fn get_positional(&self, name: &str) -> &[String] {
        match self.values.get(name) {
            None => panic!("Missing positional flag value for '{}'", name),
            Some(v) => match v {
                &Value::Repeated(ref vs) => vs.as_slice(),
                _ => panic!("Flag '{}' is not a positional flag", name),
            },
        }
    }

    /// This function looks up a positional flag's values, returning the only
    /// value in the list. This is most useful for non-variadic positional
    /// flags, which are always guaranteed to have exactly one value. This
    /// function panics if 0 or more than 1 value was found, or if the flag with
    /// the given name is of the wrong type.
    pub fn get_positional_single(&self, name: &str) -> &str {
        let vs = self.get_positional(name);
        if vs.len() > 1 {
            panic!(
                "Positional flag '{}' has more than one associated value",
                name
            );
        }

        match vs.first() {
            None => panic!("Positional flag '{}' has an empty list of values", name),
            Some(v) => v.as_str(),
        }
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
