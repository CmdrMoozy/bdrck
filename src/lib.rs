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

#![deny(
    anonymous_parameters,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces
)]
#![warn(bare_trait_objects, unreachable_pub, unused_qualifications)]

//! bdrck is a crate which contains some basic foundational tools. In general,
// the intent is to provide the kind of utilties which might be found in std
// some day, which are useful for most or all Rust programs.

extern crate chrono;
extern crate data_encoding;
extern crate errno;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate rmp_serde as msgpack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sodiumoxide;

/// The configuration module contains utilities for persisting application
/// configuration to disk.
pub mod configuration;
/// crypto contains some basic cryptographic primitives, built largely on top of
/// NaCl, which are generally useful for any program which performs crypto ops.
pub mod crypto;
/// error defines error types specific to bdrck, which properly aggregates
/// errors from all of bdrck's dependencies.
pub mod error;
/// flags defines a library for command-line argument parsing.
pub mod flags;
/// fs provides various utilities for interacting with the filesystem.
pub mod fs;
/// http provides a really thin HTTP client wrapper around reqwest. The main
/// value-add is the addition of a mechanism for recording HTTP sessions, which
/// can be used for generating data for unit tests and then replaying it during
/// the test so we can verify the client's behavior given previously observed
/// server behavior.
pub mod http;
/// logging provides Logger implementations suitable for either command-line
/// applications or serving daemons.
pub mod logging;
/// net provides additional network-related utilities, on top of what is
/// available in std.
pub mod net;
/// testing provides utilities which are useful for unit testing real production
/// code.
pub mod testing;

#[cfg(test)]
mod tests;

lazy_static! {
    static ref INIT_STATUS: ::std::sync::Mutex<bool> = ::std::sync::Mutex::new(false);
}

/// This function must be called before calling any other library code, or else
/// undefined behavior (thread safety problems in particular) may result. This
/// is due to underlying C library dependencies.
pub fn init() -> ::error::Result<()> {
    let mut lock = INIT_STATUS.lock().unwrap();
    if *lock {
        return Ok(());
    }

    if !::sodiumoxide::init().is_ok() {
        return Err(::error::Error::Internal(format_err!(
            "Initializing cryptographic dependencies failed"
        )));
    }

    *lock = true;
    Ok(())
}
