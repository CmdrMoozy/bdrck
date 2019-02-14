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

use failure::Fail;

/// Error represents the various errors which can come up while parsing
/// command-line flags.
#[derive(Fail, Debug)]
pub enum Error {
    /// An internal unrecoverable error, usually due to some underlying library.
    #[fail(display = "{}", _0)]
    Internal(::failure::Error),
    /// Errors akin to EINVAL - essentially, an argument passed into a function
    /// was invalid in some way..
    #[fail(display = "{}", _0)]
    InvalidArgument(::failure::Error),
    /// An I/O error, generally encountered when interacting with the
    /// filesystem.
    #[fail(display = "{}", _0)]
    Io(#[cause] ::std::io::Error),
    /// An awkward hack; this error exists to use String's FromStr impl, but
    /// this operation won't actually ever fail.
    #[fail(display = "{}", _0)]
    StringParse(#[cause] ::std::string::ParseError),
    /// An error of an unknown type occurred. Generally this comes from some
    /// dependency or underlying library, in a case where it's difficult to tell
    /// exactly what kind of problem occurred.
    #[fail(display = "{}", _0)]
    Unknown(::failure::Error),
    /// A flaggy_values error.
    #[fail(display = "{}", _0)]
    Values(::failure::Error),
}

impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<::std::string::ParseError> for Error {
    fn from(e: ::std::string::ParseError) -> Self {
        Error::StringParse(e)
    }
}

impl From<::flaggy_values::error::ValueError> for Error {
    fn from(e: ::flaggy_values::error::ValueError) -> Self {
        Error::Values(::failure::format_err!("{}", e))
    }
}

/// A Result type which uses flaggy's internal Error type.
pub type Result<T> = ::std::result::Result<T, Error>;
