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

/// Type denotes the particular type of flag a Spec structure describes. It
/// also contains extra metadata about the flag, if applicable for that type.
#[derive(Clone, Debug)]
pub(crate) enum Type {
    /// A required flag, meaning it must have some string value (possibly its
    /// default value) after command-line flags have been parsed, or an error
    /// has occurred.
    Required {
        /// The default value to give this flag if the user doesn't specify one.
        /// This is optional, in which case it is an error for the user not to
        /// specify a value explicitly.
        default_value: Option<String>,
    },
    /// An optional flag - the value is treated the same way as a Required flag,
    /// except it is *not* considered an error if there is no value associated
    /// with it after command-line flags have been parsed.
    Optional,
    /// A boolean flag, which may either be on or off.
    Boolean,
    /// A positional flag, which, unlike all of the other flag types, must not
    /// have its name specified explicitly (e.g. "--flag"), but which is
    /// identified purely by its position in the list of command-line arguments.
    Positional {
        /// The default value(s) for this flag, if any.
        default_value: Option<Vec<String>>,
        /// Whether or not this flag is variadic. Variadic flags greedily scoop
        /// up *all* of the remaining positional values at the end of the list
        /// of command-line arguments.
        ///
        /// Note that because of this, it only makes sense for *one* flag to be
        /// variadic, and it must always be the *last* Positional flag.
        is_variadic: bool,
    },
}

/// Spec describes a flag, in such a way that the parser can correctly identify
/// it in the set of arguments given on the command-line.
#[derive(Clone, Debug)]
pub struct Spec {
    /// The name of this flag. For non-positional flags, this must be used to
    /// identify its value, e.g. like "--flag=value".
    name: String,
    /// The help string to print out for this flag when applicable.
    help: String,
    /// The optional short name for this flag - e.g., for a flag named "flag",
    /// it may be convenient to let users alternatively identify it by "--f".
    short_name: Option<char>,
    /// The Type of flag, which identifies how its value should be identified,
    /// how it should be parsed, etc.
    flag_type: Type,
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

    /// Constructs a Spec which describes a positional flag. Flags of this type
    /// are not looked up by name after a "-" or "--" character, but instead
    /// are parsed purely by their position in the list of command-line
    /// arguments.
    ///
    /// This also means that the order in which positional flags are added to a
    /// Specs structure matters for parsing.
    ///
    /// A positional flag is variadic if it should be able to collect more than
    /// one value (e.g., for a command which takes a list of files to process
    /// of unspecified length).
    pub fn positional(
        name: &str,
        help: &str,
        mut default_value: Option<&[&str]>,
        is_variadic: bool,
    ) -> Result<Spec> {
        if let Some(dvs) = default_value {
            if dvs.len() > 1 && !is_variadic {
                return Err(Error::InvalidArgument(format!(
                    "Only variadic positional arguments can have multiple default values"
                )));
            }

            if dvs.is_empty() {
                default_value = None;
            }
        }

        Ok(Spec {
            name: name.to_owned(),
            help: help.to_owned(),
            short_name: None,
            flag_type: Type::Positional {
                default_value: default_value.map(|vs| vs.iter().map(|&v| v.to_owned()).collect()),
                is_variadic: is_variadic,
            },
        })
    }

    /// Returns true if this Spec describes a boolean flag. This function is
    /// needed because boolean flags are treated differently in some ways
    /// during parsing.
    pub(crate) fn is_boolean(&self) -> bool {
        match self.flag_type {
            Type::Boolean => true,
            _ => false,
        }
    }

    /// Returns true if this Spec describes a positional flag (that is, a flag
    /// which is interpreted by its position in the command-line arguments, not
    /// by the name it is given in the command-line arguments.
    pub(crate) fn is_positional(&self) -> bool {
        match self.flag_type {
            Type::Positional { .. } => true,
            _ => false,
        }
    }

    /// Returns true if this Spec describes a named flag. This is equivalent to
    /// !is_positional().
    pub(crate) fn is_named(&self) -> bool {
        !self.is_positional()
    }

    /// Returns true if this Spec describes a flag which has a default value
    /// (that is, one which we will still store a value for even if it does not
    /// appear in the command-line arguments).
    pub(crate) fn has_default_value(&self) -> bool {
        match self.flag_type {
            Type::Required { ref default_value } => default_value.is_some(),
            Type::Boolean => true,
            Type::Positional {
                ref default_value, ..
            } => default_value.is_some(),
            _ => false,
        }
    }

    /// Returns the number of default values this flag has. For most flag types,
    /// this will be either 0 or 1, but some flag types may support multiple
    /// default values.
    fn default_value_len(&self) -> usize {
        match self.flag_type {
            Type::Positional {
                ref default_value, ..
            } => match *default_value {
                None => 0,
                Some(ref dvs) => dvs.len(),
            },
            _ => match self.has_default_value() {
                false => 0,
                true => 1,
            },
        }
    }

    /// A flag is variadic if it can collect more than one value during parsing.
    pub(crate) fn is_variadic(&self) -> bool {
        match self.flag_type {
            Type::Positional { is_variadic, .. } => is_variadic,
            _ => false,
        }
    }

    /// Returns true if this Spec must have an associated value (either one
    /// specified or a default) after parsing command line arguments.
    pub(crate) fn is_required(&self) -> bool {
        if self.has_default_value() {
            return true;
        }
        match self.flag_type {
            Type::Required { .. } => true,
            _ => false,
        }
    }

    /// Returns this flag's full name (i.e., not the short name).
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the human-readable help text for this flag.
    pub fn get_help(&self) -> &str {
        self.help.as_str()
    }

    /// Returns this flag's short name, if it has one.
    pub fn get_short_name(&self) -> Option<char> {
        self.short_name.clone()
    }

    /// Returns this flag's type.
    pub(crate) fn get_flag_type(&self) -> &Type {
        &self.flag_type
    }

    /// Returns the default value for this Type::Required Spec, or None if it
    /// either is some other type of Spec or does not have a default value.
    pub(crate) fn get_required_default_value(&self) -> Option<&str> {
        match self.flag_type {
            Type::Required { ref default_value } => default_value.as_ref().map(|dv| dv.as_str()),
            _ => None,
        }
    }
}

/// Specs is simply a list of flag Spec structures. Whereas each Spec defines
/// the properties of a single flag, Specs defines the properties of multiple
/// flags - generally, all of the flags assocaited with a single Command.
#[derive(Clone, Debug)]
pub struct Specs {
    specs: Vec<Spec>,
}

impl Specs {
    /// Construct a new Specs structure from the given complete list of flag
    /// Spec structures.
    ///
    /// This may return an error if the Spec structures, taken together, are
    /// invalid (for example, if *multiple* variadic flags are defined).
    pub fn new(specs: Vec<Spec>) -> Result<Specs> {
        if !specs
            .iter()
            .filter(|s| s.is_positional())
            .skip_while(|s| !s.has_default_value())
            .all(|s| s.has_default_value())
        {
            return Err(Error::InvalidArgument(format!(
                "Positional flags after the first one with a default must also have defaults"
            )));
        }

        if !specs
            .iter()
            .rev()
            .skip_while(|s| !s.is_positional())
            .next()
            .map_or(true, |s| s.is_variadic() || s.default_value_len() <= 1)
        {
            return Err(Error::InvalidArgument(format!(
                "The last positional flag can only have multiple default values if it is variadic"
            )));
        }

        Ok(Specs { specs: specs })
    }

    /// Returns an Iterator over the Spec structures this Specs contains.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Spec> {
        self.specs.iter()
    }

    /// Given an iterator over a collection of Specs, locate the first Spec
    /// which matches the given name. The given name might either be a short
    /// name or a long name, depending on how it was specified on the command
    /// line.
    pub(crate) fn find_named_spec(&self, name: &str) -> Option<&Spec> {
        let mut result: Option<&Spec> = None;
        for s in &self.specs {
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
}
