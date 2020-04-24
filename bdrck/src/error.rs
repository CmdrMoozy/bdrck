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

/// Error is a structure which denotes all of the possible kinds of errors bdrck
/// can produce, including errors from any of its underlying dependencies.
#[derive(Debug, Error)]
pub enum Error {
    /// An error encountered while performing a cryptographic operation.
    #[error("cryptographic operation failed: {0}")]
    Crypto(String),
    /// An error encountered while trying to interact with environment
    /// variables.
    #[error("{0}")]
    EnvVar(#[from] std::env::VarError),
    /// An error decoding bytes as UTF-8 text.
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    /// An error decoding bytes as UTF-8 text (except for `str` instead of `String`).
    #[error("{0}")]
    FromUtf8Str(#[from] std::str::Utf8Error),
    /// An error encountered in trying to decode a hex string to the bytes it
    /// represents.
    #[cfg(feature = "data-encoding")]
    #[error("{0}")]
    HexDecode(#[from] data_encoding::DecodeError),
    /// An error originating in HTTP client code.
    #[cfg(feature = "reqwest")]
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    /// An HTTP request failed, despite multiple retries.
    #[error("HTTP request failed despite retries: {0}")]
    HttpRetry(String),
    /// This error indicates that we were reading some input, and we encountered
    /// too many bytes (e.g. because there was an upper bound on how much we
    /// were willing to read).
    #[error("input too big: {0}")]
    InputTooBig(String),
    /// An internal unrecoverable error, usually due to some underlying library.
    #[error("internal error: {0}")]
    Internal(String),
    /// Errors akin to EINVAL - essentially, an argument passed into a function
    /// was invalid in some way..
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// An I/O error, generally encountered when interacting with the
    /// filesystem.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// An error encountered while serializing or deserializing JSON.
    #[cfg(feature = "serde_json")]
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// An error encountered when decoding a serialized message.
    #[cfg(feature = "rmp-serde")]
    #[error("{0}")]
    MsgDecode(#[from] rmp_serde::decode::Error),
    /// An error encountered when encoding a struct to a serialized message.
    #[cfg(feature = "rmp-serde")]
    #[error("{0}")]
    MsgEncode(#[from] rmp_serde::encode::Error),
    /// Errors akin to ENOENT - something like e.g. "file not found", although
    /// this is not necessarily *always* about files.
    #[error("not found: {0}")]
    NotFound(String),
    /// An error where some data returned by an underlying library call
    /// contained a NUL byte ('\0'), in a context where such a thing is invalid.
    #[error("{0}")]
    Nul(#[from] std::ffi::NulError),
    /// An error encountered when trying to parse an integer from a string.
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    /// An error encountered when trying to parse an IP address from a string.
    #[error("{0}")]
    ParseIpAddr(#[from] std::net::AddrParseError),
    /// A precondition error, which basically amounts to a function being called
    /// when one or more of its preconditions were not satisfied.
    #[error("precondition not satisfied: {0}")]
    Precondition(String),
    /// An error encountered in either parsing or applying a regular expression.
    #[cfg(feature = "regex")]
    #[error("{0}")]
    Regex(#[from] regex::Error),
    /// An error encountered when attempting to set the global Logger
    /// implementation.
    #[cfg(feature = "log")]
    #[error("{0}")]
    SetLogger(#[from] log::SetLoggerError),
    /// An awkward hack; this error exists to use String's FromStr impl, but
    /// this operation won't actually ever fail.
    #[error("{0}")]
    StringParse(#[from] std::string::ParseError),
    /// An error in decoding a URL.
    #[cfg(feature = "url")]
    #[error("{0}")]
    Url(#[from] url::ParseError),
}

/// A Result type which uses bdrck's internal Error type.
pub type Result<T> = std::result::Result<T, Error>;
