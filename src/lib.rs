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
    anonymous_parameters, trivial_casts, trivial_numeric_casts, unused_extern_crates,
    unused_import_braces
)]
#![warn(bare_trait_objects, unreachable_pub, unused_qualifications)]

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
extern crate rmp_serde as msgpack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate sodiumoxide;

pub mod configuration;
pub mod crypto;
pub mod error;
pub mod flags;
pub mod fs;
pub mod logging;
pub mod net;
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
