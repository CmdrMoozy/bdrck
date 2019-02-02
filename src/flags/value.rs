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

use crate::flags::spec::{Spec, Specs, Type};
use flags_values::error::{ValueError, ValueResult};
use flags_values::value::{Value, Values};
use std::collections::HashMap;
use std::iter::Peekable;

/// Returns a collection containing all default values from the given flag
/// Specs.
fn get_default_values<'a>(specs: &Specs) -> HashMap<String, Value> {
    specs
        .iter()
        .filter(|s| s.has_default_value())
        .map(|s| -> (String, Value) {
            match s.get_flag_type() {
                Type::Required { ref default_value } => (
                    s.get_name().to_owned(),
                    Value::Single(default_value.as_ref().unwrap().clone()),
                ),
                Type::Boolean => (s.get_name().to_owned(), Value::Boolean(false)),
                Type::Positional {
                    ref default_value, ..
                } => (
                    s.get_name().to_owned(),
                    Value::Repeated(default_value.as_ref().unwrap().clone()),
                ),
                _ => panic!(
                    "Default value lookup for {:?} not implemented",
                    s.get_flag_type()
                ),
            }
        })
        .collect()
}

/// Return the boolean interpretation of a string, or an error if the string
/// isn't recognized as a valid boolean value.
fn parse_bool(value: &str) -> ValueResult<bool> {
    match value.trim().to_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => return Err(ValueError::BadBoolean(value.to_owned())),
    }
}

/// Constructs a new Value for a named flag. Note that named flags can never
/// have repeated values, so this function only handles the Single and
/// Boolean cases.
fn new_named_flag_value(spec: &Spec, value: Option<String>) -> ValueResult<Value> {
    Ok(match spec.is_boolean() {
        false => Value::Single(match value {
            None => return Err(ValueError::MissingValue(spec.get_name().to_owned())),
            Some(value) => value,
        }),
        true => Value::Boolean(match value {
            None => true,
            Some(value) => parse_bool(value.as_str())?,
        }),
    })
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
    value: Option<String>,
    spec: &'a Spec,
}

impl<'a> NamedFlagSpec<'a> {
    /// Constructs a new NamedFlagSpec by parsing the given flag argument. The
    /// given string must start with "-" or "--", and it may optionally contain
    /// an associated value after an "=" character. The part of the string
    /// after the hyphens but before the "=" is considered to be the flag name,
    /// and is used to look up the associated Spec. This name may be either the
    /// short or long name for the flag.
    fn new<'b>(specs: &'a Specs, flag: &'b str) -> ValueResult<NamedFlagSpec<'a>> {
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
            None => return Err(ValueError::UnknownFlag(name.to_owned())),
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
    name: String,
    is_variadic: bool,
}

/// Parse the next named flag from the given iterator over this program's
/// command-line arguments. If there are no more named flags, returns None. If
/// a flag was found but some error occured in parsing its name or value, then
/// an error is returned.
fn parse_next_named_flag<'a, 'b, I: Iterator<Item = &'b String>>(
    specs: &'a Specs,
    args: &mut Peekable<I>,
) -> ValueResult<Option<ParsedNamedFlag>> {
    let flag: &'b String = match args.peek() {
        Some(p) => {
            if p.starts_with('-') {
                p
            } else {
                return Ok(None);
            }
        }
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
        value: new_named_flag_value(&spec.spec, spec.value)?,
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
    fn new(specs: &'a Specs, args: Peekable<I>) -> ValueIterator<'a, 'b, I> {
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
    type Item = ValueResult<(String, Value)>;

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
                }
            },
        }
    }
}

pub(crate) fn build_values<'a, 'b, I: Iterator<Item = &'b String>>(
    specs: &'a Specs,
    args: Peekable<I>,
) -> ValueResult<Values> {
    let default_values = get_default_values(specs);
    let values: ValueResult<HashMap<String, Value>> = ValueIterator::new(specs, args).collect();
    let values = Values::new(default_values, values?);

    for s in specs.iter() {
        if s.is_required() && values.get(s.get_name()).is_none() {
            return Err(ValueError::MissingFlag(s.get_name().to_owned()));
        }
    }

    Ok(values)
}
