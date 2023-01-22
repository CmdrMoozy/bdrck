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

use crate::crypto::secret::Secret;
use halite_sys;
use libc::c_void;

/// Fill the given buffer with random bytes. This function is guaranteed to be thread safe and
/// cryptographically secure. In other words, it's fine to use this for generating passwords, key
/// material, etc.
pub fn randombytes_into(buf: &mut [u8]) {
    debug_assert!(crate::init_done());
    unsafe {
        halite_sys::randombytes_buf(buf.as_mut_ptr() as *mut c_void, buf.len());
    }
}

/// A simple wrapper around `randombytes_into` which fills a `Secret` instead of a simple byte
/// buffer.
pub fn randombytes_into_secret(s: &mut Secret) {
    randombytes_into(unsafe { s.as_mut_slice() });
}
