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

/// Error is a structure which denotes all of the possible kinds of errors bdrck
/// can produce, including errors from any of its underlying dependencies.
#[derive(Fail, Debug)]
pub enum Error {
    /// An error encountered while trying to interact with environment
    /// variables.
    #[fail(display = "{}", _0)]
    EnvVar(#[cause] ::std::env::VarError),
    /// An error encountered in trying to decode a hex string to the bytes it
    /// represents.
    #[fail(display = "{}", _0)]
    HexDecode(#[cause] ::data_encoding::DecodeError),
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
    /// An error encountered when decoding a serialized message.
    #[fail(display = "{}", _0)]
    MsgDecode(#[cause] ::msgpack::decode::Error),
    /// An error encountered when encoding a struct to a serialized message.
    #[fail(display = "{}", _0)]
    MsgEncode(#[cause] ::msgpack::encode::Error),
    /// Errors akin to ENOENT - something like e.g. "file not found", although
    /// this is not necessarily *always* about files.
    #[fail(display = "{}", _0)]
    NotFound(::failure::Error),
    /// An error where some data returned by an underlying library call
    /// contained a NUL byte ('\0'), in a context where such a thing is invalid.
    #[fail(display = "{}", _0)]
    Nul(#[cause] ::std::ffi::NulError),
    /// An error encountered when trying to parse an integer from a string.
    #[fail(display = "{}", _0)]
    ParseInt(#[cause] ::std::num::ParseIntError),
    /// An error encountered when trying to parse an IP address from a string.
    #[fail(display = "{}", _0)]
    ParseIpAddr(#[cause] ::std::net::AddrParseError),
    /// A precondition error, which basically amounts to a function being called
    /// when one or more of its preconditions were not satisfied.
    #[fail(display = "{}", _0)]
    Precondition(::failure::Error),
    /// An error encountered in either parsing or applying a regular expression.
    #[fail(display = "{}", _0)]
    Regex(#[cause] ::regex::Error),
    /// An error encountered when attempting to set the global Logger
    /// implementation.
    #[fail(display = "{}", _0)]
    SetLogger(#[cause] ::log::SetLoggerError),
    /// An error of an unknown type occurred. Generally this comes from some
    /// dependency or underlying library, in a case where it's difficult to tell
    /// exactly what kind of problem occurred.
    #[fail(display = "{}", _0)]
    Unknown(::failure::Error),
}

impl From<::std::env::VarError> for Error {
    fn from(e: ::std::env::VarError) -> Self {
        Error::EnvVar(e)
    }
}

impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<::msgpack::decode::Error> for Error {
    fn from(e: ::msgpack::decode::Error) -> Self {
        Error::MsgDecode(e)
    }
}

impl From<::msgpack::encode::Error> for Error {
    fn from(e: ::msgpack::encode::Error) -> Self {
        Error::MsgEncode(e)
    }
}

impl From<::std::ffi::NulError> for Error {
    fn from(e: ::std::ffi::NulError) -> Self {
        Error::Nul(e)
    }
}

impl From<::std::num::ParseIntError> for Error {
    fn from(e: ::std::num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<::std::net::AddrParseError> for Error {
    fn from(e: ::std::net::AddrParseError) -> Self {
        Error::ParseIpAddr(e)
    }
}

impl From<::regex::Error> for Error {
    fn from(e: ::regex::Error) -> Self {
        Error::Regex(e)
    }
}

impl From<::log::SetLoggerError> for Error {
    fn from(e: ::log::SetLoggerError) -> Self {
        Error::SetLogger(e)
    }
}

/// A Result type which uses bdrck's internal Error type.
pub type Result<T> = ::std::result::Result<T, Error>;
