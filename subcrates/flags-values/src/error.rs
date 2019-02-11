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

pub enum ValueError {
    /// A malformed boolean value was found.
    BadBoolean(String),
    /// A `Command` callback parameter was of the wrong type, so flag value
    /// passing failed.
    CallbackParameter(String),
    /// No command was specified.
    MissingCommand,
    /// A required flag was not provided.
    MissingFlag(String),
    /// A flag was provided with no associated value.
    MissingValue(String),
    /// An unrecognized command was provided.
    UnknownCommand(String),
    /// An unrecognized flag was provided.
    UnknownFlag(String),
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueError::BadBoolean(v) => write!(f, "Invalid boolean value '{}'", v),
            ValueError::CallbackParameter(msg) => {
                write!(f, "Incorrect Command callback parameter: {}", msg)
            }
            ValueError::MissingCommand => write!(f, "No command specified"),
            ValueError::MissingFlag(n) => write!(f, "No value provided for required flag '{}'", n),
            ValueError::MissingValue(n) => write!(f, "Flag '{}' provided without any value", n),
            ValueError::UnknownCommand(c) => write!(f, "Unknown command '{}'", c),
            ValueError::UnknownFlag(n) => write!(f, "Unknown flag '{}'", n),
        }
    }
}

pub type ValueResult<T> = Result<T, ValueError>;
