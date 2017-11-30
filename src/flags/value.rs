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

use error::*;
use flags::spec::{Spec, Specs, Type};
use std::collections::HashMap;
use std::iter::{FromIterator, Peekable};
use std::str::FromStr;

/// Returns a collection containing all default values from the given flag
/// Specs.
fn get_default_values<'a>(specs: &Specs) -> HashMap<String, Value> {
    specs
        .iter()
        .filter(|s| s.has_default_value())
        .map(|s| -> (String, Value) {
            match s.flag_type {
                Type::Required { ref default_value } => (
                    s.name.clone(),
                    Value::Single(default_value.as_ref().unwrap().clone()),
                ),
                Type::Boolean => (s.name.clone(), Value::Boolean(false)),
                Type::Positional {
                    ref default_value, ..
                } => (
                    s.name.clone(),
                    Value::Repeated(default_value.as_ref().unwrap().clone()),
                ),
                _ => panic!("Default value lookup for {:?} not implemented", s.flag_type),
            }
        })
        .collect()
}

/// Return the boolean interpretation of a string, or an error if the string
/// isn't recognized as a valid boolean value.
fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Invalid boolean value '{}'", value),
    }
}

/// A Value is the value associated with a given flag. The type of the value
/// is different depending on the type of flag it is associated with.
#[derive(Debug, Eq, PartialEq)]
pub enum Value {
    Single(String),
    Boolean(bool),
    Repeated(Vec<String>),
}

impl Value {
    /// Constructs a new Value for a named flag. Note that named flags can never
    /// have repeated values, so this function only handles the Single and
    /// Boolean cases.
    pub fn new_named_flag_value(spec: &Spec, value: Option<String>) -> Result<Value> {
        Ok(match spec.is_boolean() {
            false => Value::Single(match value {
                None => bail!("Missing value for flag '{}'", spec.get_name()),
                Some(value) => value,
            }),
            true => Value::Boolean(match value {
                None => true,
                Some(value) => parse_bool(value.as_str())?,
            }),
        })
    }
}

/// A ParsedNamedFlag is a flag name and its associated value, after it has been
/// parsed from the program's command-line arguments. Note that this structure
/// is only used for named flags, not positional flags.
struct ParsedNamedFlag {
    name: String,
    value: Value,
}

/// A NamedFlagSpec describes a named flag, as parsed / extracted from an
/// iterator over command-line arguments. It may contain a value, if the
/// flag was passed as '--name=value', but it may not if the value was
/// passed separately ('--name', 'value').
struct NamedFlagSpec<'a> {
    pub value: Option<String>,
    pub spec: &'a Spec,
}

impl<'a> NamedFlagSpec<'a> {
    /// Constructs a new NamedFlagSpec by parsing the given flag argument. The
    /// given string must start with "-" or "--", and it may optionally contain
    /// an associated value after an "=" character. The part of the string
    /// after the hyphens but before the "=" is considered to be the flag name,
    /// and is used to look up the associated Spec. This name may be either the
    /// short or long name for the flag.
    pub fn new<'b>(specs: &'a Specs, flag: &'b str) -> Result<NamedFlagSpec<'a>> {
        let trimmed = if flag.starts_with("--") {
            &flag[2..]
        } else {
            &flag[1..]
        };
        let equals_idx = trimmed.rfind('=');
        let name = equals_idx.map_or(trimmed, |ei| &trimmed[0..ei]);
        let value = equals_idx.map_or(None, |ei| Some((&trimmed[ei + 1..]).to_owned()));

        let spec: &'a Spec = match specs.find_named_spec(name) {
            Some(s) => s,
            None => bail!("Unrecognized flag '{}'", name),
        };

        Ok(NamedFlagSpec {
            value: value,
            spec: spec,
        })
    }
}

/// A PositionalFlagSpec encapsulates the metadata about a positional flag which
/// is necessary to parse it from command-line arguments.
struct PositionalFlagSpec {
    pub name: String,
    pub is_variadic: bool,
}

/// Parse the next named flag from the given iterator over this program's
/// command-line arguments. If there are no more named flags, returns None. If
/// a flag was found but some error occured in parsing its name or value, then
/// an error is returned.
fn parse_next_named_flag<'a, 'b, I: Iterator<Item = &'b String>>(
    specs: &'a Specs,
    args: &mut Peekable<I>,
) -> Result<Option<ParsedNamedFlag>> {
    let flag: &'b String = match args.peek() {
        Some(p) => if p.starts_with('-') {
            p
        } else {
            return Ok(None);
        },
        None => return Ok(None),
    };
    args.next();

    let mut spec = NamedFlagSpec::new(specs, flag)?;

    // Search for the value in the next parameter, if this flag is not a
    // boolean flag. For boolean flags, because explicit values are optional,
    // it is ambiguous whether the next arg is our value or a positional flag.
    if !spec.spec.is_boolean() {
        let next_arg_is_value = args.peek().map_or(false, |v| !v.starts_with('-'));
        if next_arg_is_value && spec.value.is_none() {
            spec.value = Some(args.next().unwrap().clone());
        }
    }

    Ok(Some(ParsedNamedFlag {
        name: spec.spec.get_name().to_owned(),
        value: Value::new_named_flag_value(&spec.spec, spec.value)?,
    }))
}

/// ValueIterator defines an Iterator which parses a full set of flag Specs from
/// a given Iterator over command-line arguments.
struct ValueIterator<'a, 'b, I: Iterator<Item = &'b String>> {
    specs: &'a Specs,
    args: Peekable<I>,
    /// Indicates whether or not we have parsed all of the named flags out of
    /// the given args Iterator. Since all positional args must come after any
    /// named args, this indicates whether we should be parsing one or the
    /// other.
    finished_named_flags: bool,
    /// The positional arguments to parse out of the command-line arguments, in
    /// reverse order (so pop() will return them in order).
    positional_specs: Vec<PositionalFlagSpec>,
}

impl<'a, 'b, I: Iterator<Item = &'b String>> ValueIterator<'a, 'b, I> {
    /// Initializes a new ValueIterator over the given command-line argument
    /// Iterator, which is ready to start parsing flags.
    pub fn new(specs: &'a Specs, args: Peekable<I>) -> ValueIterator<'a, 'b, I> {
        ValueIterator {
            specs: specs,
            args: args,
            finished_named_flags: false,
            positional_specs: specs
                .iter()
                .filter_map(|s| match s.is_positional() {
                    false => None,
                    true => Some(PositionalFlagSpec {
                        name: s.get_name().to_owned(),
                        is_variadic: s.is_variadic(),
                    }),
                })
                .rev()
                .collect(),
        }
    }
}

impl<'a, 'b, I: Iterator<Item = &'b String>> Iterator for ValueIterator<'a, 'b, I> {
    type Item = Result<(String, Value)>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.finished_named_flags {
            match parse_next_named_flag(self.specs, &mut self.args) {
                Ok(parsed_flag) => match parsed_flag {
                    Some(parsed_flag) => return Some(Ok((parsed_flag.name, parsed_flag.value))),
                    None => self.finished_named_flags = true,
                },
                Err(e) => return Some(Err(e)),
            }
        }

        match self.positional_specs.pop() {
            None => None,
            Some(spec) => match spec.is_variadic {
                false => match self.args.next() {
                    None => None,
                    Some(value) => Some(Ok((spec.name, Value::Repeated(vec![value.clone()])))),
                },
                true => {
                    let mut values = vec![];
                    while let Some(value) = self.args.next() {
                        values.push(value.clone());
                    }
                    Some(Ok((spec.name, Value::Repeated(values))))
                },
            },
        }
    }
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
    /// Constructs a new Values by parsing the flag values out of the given
    /// Iterator over command-line arguments, and using the given flag Specs.
    pub fn new<'a, 'b, I: Iterator<Item = &'b String>>(
        specs: &'a Specs,
        args: Peekable<I>,
    ) -> Result<Values> {
        let default_values = get_default_values(specs);
        let values: Result<HashMap<String, Value>> = ValueIterator::new(specs, args).collect();
        let mut values: HashMap<String, Value> = values?;
        for (name, value) in default_values.into_iter() {
            values.entry(name).or_insert(value);
        }

        for s in specs.iter() {
            if s.is_required() && !values.contains_key(&s.name) {
                bail!("Unexpected missing value for flag '{}'", s.name);
            }
        }

        Ok(Values { values: values })
    }

    /// Looks up a generic flag value, which may or may not be present. This
    /// function is guaranteed not to panic, but error handling and type
    /// matching is left up to the caller do deal with at runtime.
    pub fn get(&self, name: &str) -> Option<&Value> { self.values.get(name) }

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
    fn from(values: HashMap<String, Value>) -> Self { Values { values: values } }
}

impl FromIterator<(String, Value)> for Values {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        let values: HashMap<String, Value> = iter.into_iter().collect();
        values.into()
    }
}
