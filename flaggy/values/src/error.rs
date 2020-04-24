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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValueError {
    /// A malformed boolean value was found.
    #[error("Invalid boolean value '{0}'")]
    BadBoolean(String),
    /// A `Command` callback parameter was of the wrong type, so flag value
    /// passing failed.
    #[error("Incorrect command callback parameter: {0}")]
    CallbackParameter(String),
    /// No command was specified.
    #[error("No command specified")]
    MissingCommand,
    /// A required flag was not provided.
    #[error("No value provided for required flag '{0}'")]
    MissingFlag(String),
    /// A flag was provided with no associated value.
    #[error("Flag '{0}' provided without any value")]
    MissingValue(String),
    /// An unrecognized command was provided.
    #[error("Unknown command '{0}'")]
    UnknownCommand(String),
    /// An unrecognized flag was provided.
    #[error("Unknown flag '{0}'")]
    UnknownFlag(String),
}

pub type ValueResult<T> = Result<T, ValueError>;
