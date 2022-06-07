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

use crate::crypto::compat;
use crate::crypto::secret::Secret;
use crate::crypto::util::randombytes_into;
use crate::error::*;
use halite_sys;
use libc::{c_char, c_ulonglong};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// This module uses sha512, which produces 64 byte digests.
pub const DIGEST_BYTES: usize = halite_sys::crypto_hash_sha512_BYTES as usize;
/// scryptsalsa208sha256 uses 32 byte salts.
pub const SALT_BYTES: usize = halite_sys::crypto_pwhash_scryptsalsa208sha256_SALTBYTES as usize;

/// Safe ops_limit base line for password-based key derivation, for interactive
/// password hashing.
pub const OPS_LIMIT_INTERACTIVE: usize =
    halite_sys::crypto_pwhash_scryptsalsa208sha256_OPSLIMIT_INTERACTIVE as usize;
/// ops_limit for highly sensitive data.
pub const OPS_LIMIT_SENSITIVE: usize =
    halite_sys::crypto_pwhash_scryptsalsa208sha256_OPSLIMIT_SENSITIVE as usize;
/// Safe mem_limit base line for password-based key derivation, for interactive
/// password hashing.
pub const MEM_LIMIT_INTERACTIVE: usize =
    halite_sys::crypto_pwhash_scryptsalsa208sha256_MEMLIMIT_INTERACTIVE as usize;
/// mem_limit for highly sensitive data.
pub const MEM_LIMIT_SENSITIVE: usize =
    halite_sys::crypto_pwhash_scryptsalsa208sha256_MEMLIMIT_SENSITIVE as usize;

/// A digest is a cryptographic hash of some arbitrary input data, with the goal
/// of identifying it or detecting changes with high probability.
#[derive(Clone, Eq, PartialEq)]
pub struct Digest([u8; DIGEST_BYTES]);

// Implement by hand instead of derive for slightly nicer output (no struct name).
impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0.as_ref())
    }
}

// Unfortunately has to be implemented manually, as derive doesn't work for large fixed-size arrays.
impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(DIGEST_BYTES))?;
        for element in self.0.as_ref() {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

// Unfortunately has to be implemented manually, as derive doesn't work for large fixed-size arrays.
impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> ::std::result::Result<Digest, D::Error> {
        struct DigestVisitor;

        impl<'vde> Visitor<'vde> for DigestVisitor {
            type Value = Digest;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a sequence of {} bytes", DIGEST_BYTES)
            }

            fn visit_seq<A: SeqAccess<'vde>>(
                self,
                mut seq: A,
            ) -> ::std::result::Result<Self::Value, A::Error> {
                let mut digest = Digest([0; DIGEST_BYTES]);
                for i in 0..DIGEST_BYTES {
                    digest.0[i] = match seq.next_element()? {
                        Some(val) => val,
                        None => return Err(::serde::de::Error::invalid_length(i + 1, &self)),
                    };
                }
                if seq.next_element::<u8>()?.is_some() {
                    return Err(::serde::de::Error::invalid_length(DIGEST_BYTES + 1, &self));
                }
                Ok(digest)
            }
        }

        deserializer.deserialize_seq(DigestVisitor)
    }
}

impl Digest {
    /// Construct a new Digest object by hashing the given raw bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut digest = Digest([0; DIGEST_BYTES]);
        unsafe {
            halite_sys::crypto_hash_sha512(digest.0.as_mut_ptr(), data.as_ptr(), data.len() as u64);
        }
        digest
    }

    /// Construct a new Digest object by hashing the given Secret's raw bytes.
    pub fn from_secret(secret: &Secret) -> Self {
        Self::from_bytes(unsafe { secret.as_slice() })
    }
}

/// A salt is an arbitrary byte sequence which is used for password-based key
/// derivation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Salt(compat::Salt);

impl Default for Salt {
    fn default() -> Self {
        let mut s = Salt(compat::Salt::default());
        randombytes_into(&mut s.0 .0);
        s
    }
}

/// Hash the given password using the given salt and ops limits, placing the result in the given
/// buffer. Note that the length of "out" is mostly arbitrary.
///
/// The purpose of this is basically to take a password of arbitrary length, and to generate a key
/// of some other length from it.
///
/// Both the input and output here are Secrets, because the key derivation algorithm is not secret.
/// The output, the key, is ostensibly going to be used for encryption, or whatever, so it needs to
/// remain secret. The key derivation algorithm isn't secret, so the output can be trivially
/// derived from the input. Therefore, the input needs to be a Secret as well.
pub fn derive_key(
    out: &mut Secret,
    password: &Secret,
    salt: &Salt,
    ops_limit: usize,
    mem_limit: usize,
) -> Result<()> {
    if unsafe {
        halite_sys::crypto_pwhash_scryptsalsa208sha256(
            out.slice_ptr(),
            out.len() as c_ulonglong,
            password.slice_ptr() as *const c_char,
            password.len() as c_ulonglong,
            salt.0 .0.as_ptr(),
            ops_limit as c_ulonglong,
            mem_limit as c_ulonglong,
        )
    } == 0
    {
        Ok(())
    } else {
        // NOTE: We handle this error gracefully, but in reality (by inspecting the
        // libsodium source code) the only way this can actually fail is if the input
        // password is *enormous*. So, this won't really fail in practice.
        Err(Error::Internal(format!(
            "deriving key from password failed"
        )))
    }
}
