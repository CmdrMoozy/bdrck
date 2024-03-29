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

/// Utilities for command-line interfaces.
#[cfg(feature = "cli")]
pub mod cli;
/// The configuration module contains utilities for persisting application
/// configuration to disk.
#[cfg(feature = "configuration")]
pub mod configuration;
/// crypto contains some basic cryptographic primitives, built largely on top of
/// NaCl, which are generally useful for any program which performs crypto ops.
#[cfg(feature = "crypto")]
pub mod crypto;
/// error defines error types specific to bdrck, which properly aggregates
/// errors from all of bdrck's dependencies.
pub mod error;
/// fs provides various utilities for interacting with the filesystem.
#[cfg(feature = "fs")]
pub mod fs;
/// http provides a really thin HTTP client wrapper around reqwest. The main
/// value-add is the addition of a mechanism for recording HTTP sessions, which
/// can be used for generating data for unit tests and then replaying it during
/// the test so we can verify the client's behavior given previously observed
/// server behavior.
#[cfg(feature = "http")]
pub mod http;
/// io provides additional small utilities on top of std::io.
#[cfg(feature = "io")]
pub mod io;
/// net provides additional network-related utilities, on top of what is
/// available in std.
#[cfg(feature = "net")]
pub mod net;
/// testing provides utilities which are useful for unit testing real production
/// code.
#[cfg(feature = "testing")]
pub mod testing;

// Tests have significantly more dependencies than the code being tested. Don't
// bother running tests unless all features are enabled.
#[cfg(all(
    feature = "cli",
    feature = "configuration",
    feature = "crypto",
    feature = "fs",
    feature = "http",
    feature = "net",
    feature = "testing"
))]
#[cfg(test)]
mod tests;

use once_cell::sync::Lazy;

static INIT_STATUS: Lazy<::std::sync::Mutex<bool>> = Lazy::new(|| ::std::sync::Mutex::new(false));

#[cfg(feature = "halite-sys")]
fn init_nacl() -> self::error::Result<()> {
    if unsafe { halite_sys::sodium_init() } >= 0 {
        Ok(())
    } else {
        Err(error::Error::Internal(format!(
            "initializing cryptographic dependencies failed"
        )))
    }
}

#[cfg(not(feature = "halite-sys"))]
fn init_nacl() -> self::error::Result<()> {
    Ok(())
}

/// This function must be called before calling any other library code, or else
/// undefined behavior (thread safety problems in particular) may result. This
/// is due to underlying C library dependencies.
pub fn init() -> self::error::Result<()> {
    let mut lock = INIT_STATUS.lock().unwrap();
    if *lock {
        return Ok(());
    }

    init_nacl()?;

    *lock = true;
    Ok(())
}

/// Returns whether or not init() has been called.
pub fn init_done() -> bool {
    *INIT_STATUS.lock().unwrap()
}
