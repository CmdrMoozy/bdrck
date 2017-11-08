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

use std::fmt;
use std::option::Option as Optional;

/// An argument is a positional parameter. It must come after any Options
/// the command supports, and can have a default value if it is not
/// specified by the user explicitly.
///
/// The final Argument for a Command can be variadic (that is, it can
/// accept more than one value), but whether or not this is the case is a
/// property of the Command, not of the Argument (because the Argument only
/// stores a description of the argument, not its final value).
#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub help: String,
    pub default_value: Optional<Vec<String>>,
}

impl Argument {
    pub fn new(name: &str, help: &str, default_value: Optional<Vec<String>>) -> Argument {
        Argument {
            name: name.to_owned(),
            help: help.to_owned(),
            default_value: default_value,
        }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.name, self.help)?;
        if let Some(default) = self.default_value.as_ref() {
            write!(f, " [Default: {}]", &default[..].join(", "))?;
        }
        Ok(())
    }
}
