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

/// Error represents the various errors which can come up while parsing
/// command-line flags.
#[derive(Debug, Error)]
pub enum Error {
    /// An error parsing an IP address.
    #[error("{0}")]
    AddrParse(#[from] std::net::AddrParseError),
    /// An internal unrecoverable error, usually due to some underlying library.
    #[error("Internal error: {0}")]
    Internal(String),
    /// Errors akin to EINVAL - essentially, an argument passed into a function
    /// was invalid in some way..
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    /// An I/O error, generally encountered when interacting with the
    /// filesystem.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// An awkward hack; this error exists to use String's FromStr impl, but
    /// this operation won't actually ever fail.
    #[error("{0}")]
    StringParse(#[from] std::string::ParseError),
    /// A flaggy_values error.
    #[error("{0}")]
    Values(#[from] flaggy_values::error::ValueError),
}

/// A Result type which uses flaggy's internal Error type.
pub type Result<T> = std::result::Result<T, Error>;
