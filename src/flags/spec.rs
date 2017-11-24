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

/// Type denotes the particular type of flag a Spec structure describes. It
/// also contains extra metadata about the flag, if applicable for that type.
pub enum Type {
    Required { default_value: Option<String> },
    Optional,
    Boolean,
    Positional { default_value: Option<Vec<String>> },
}

/// Spec describes a flag, in such a way that the parser can correctly identify
/// it in the set of arguments given on the command-line.
pub struct Spec {
    pub name: String,
    pub help: String,
    pub short_name: Option<char>,
    pub flag_type: Type,
}

impl Spec {
    /// Constructs a Spec which describes a required named flag. This flag may
    /// have a default value, but the key point is that it must have some value
    /// after parsing is complete.
    pub fn required(
        name: &str,
        help: &str,
        short_name: Option<char>,
        default_value: Option<&str>,
    ) -> Spec {
        Spec {
            name: name.to_owned(),
            help: help.to_owned(),
            short_name: short_name,
            flag_type: Type::Required {
                default_value: default_value.map(|dv| dv.to_owned()),
            },
        }
    }

    /// Constructs a Spec which describes an optional named flag. This flag
    /// may not have a value after we are finished parsing.
    pub fn optional(name: &str, help: &str, short_name: Option<char>) -> Spec {
        Spec {
            name: name.to_owned(),
            help: help.to_owned(),
            short_name: short_name,
            flag_type: Type::Optional,
        }
    }

    /// Constructs a Spec which describes a boolean named flag. Flags of this
    /// type always have a value, and that value is either true or false,
    /// instead of being a freeform string like other flag types.
    ///
    /// If this flag is not specified at all on the command line, its "default
    /// value" is false.
    pub fn boolean(name: &str, help: &str, short_name: Option<char>) -> Spec {
        Spec {
            name: name.to_owned(),
            help: help.to_owned(),
            short_name: short_name,
            flag_type: Type::Boolean,
        }
    }

    pub fn positional(name: &str, help: &str, default_value: Option<&[&str]>) -> Spec {
        Spec {
            name: name.to_owned(),
            help: help.to_owned(),
            short_name: None,
            flag_type: Type::Positional {
                default_value: default_value.map(|vs| vs.iter().map(|&v| v.to_owned()).collect()),
            },
        }
    }

    /// Returns true if this Spec describes a positional flag (that is, a flag
    /// which is interpreted by its position in the command-line arguments, not
    /// by the name it is given in the command-line arguments.
    pub fn is_positional(&self) -> bool {
        match self.flag_type {
            Type::Positional { .. } => true,
            _ => false,
        }
    }

    /// Returns true if this Spec describes a named flag. This is equivalent to
    /// !is_positional().
    pub fn is_named(&self) -> bool { !self.is_positional() }
}

/// Given an iterator over a collection of Specs, locate the first Spec which
/// matches the given name. The given name might either be a short name or a
/// long name, depending on how it was specified on the command line.
pub fn find_named_spec<'a, I: Iterator<Item = &'a Spec>>(specs: I, name: &str) -> Option<&'a Spec> {
    let mut result: Option<&'a Spec> = None;
    for s in specs {
        if s.is_named() {
            if s.name == name {
                result = Some(s);
                break;
            } else if let Some(sn) = s.short_name {
                if name.len() == 1 && name.starts_with(sn) {
                    result = result.or_else(|| Some(s));
                }
            }
        }
    }
    result
}
