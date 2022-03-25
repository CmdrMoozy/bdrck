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

use crate::error::*;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::hash;
use sodiumoxide::crypto::pwhash;
use std::fmt;

/// This module uses sha512, which produces 64 byte digests.
pub const DIGEST_BYTES: usize = hash::DIGESTBYTES;
/// scryptsalsa208sha256 uses 32 byte salts.
pub const SALT_BYTES: usize = pwhash::SALTBYTES;

/// Safe ops_limit base line for password-based key derivation, for interactive
/// password hashing.
pub const OPS_LIMIT_INTERACTIVE: usize = pwhash::OPSLIMIT_INTERACTIVE.0;
/// ops_limit for highly sensitive data.
pub const OPS_LIMIT_SENSITIVE: usize = pwhash::OPSLIMIT_SENSITIVE.0;
/// Safe mem_limit base line for password-based key derivation, for interactive
/// password hashing.
pub const MEM_LIMIT_INTERACTIVE: usize = pwhash::MEMLIMIT_INTERACTIVE.0;
/// mem_limit for highly sensitive data.
pub const MEM_LIMIT_SENSITIVE: usize = pwhash::MEMLIMIT_SENSITIVE.0;

/// A digest is a cryptographic hash of some arbitrary input data, with the goal
/// of identifying it or detecting changes with high probability.
#[derive(Clone)]
pub struct Digest([u8; DIGEST_BYTES]);

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0.as_ref())
    }
}

impl PartialEq for Digest {
    fn eq(&self, other: &Digest) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl Eq for Digest {}

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(DIGEST_BYTES))?;
        for element in self.0.as_ref() {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

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
        Digest(hash::hash(data).0)
    }
}

/// A salt is an arbitrary byte sequence which is used for password-based key
/// derivation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Salt(pwhash::Salt);

impl Default for Salt {
    fn default() -> Self {
        Salt(pwhash::gen_salt())
    }
}

/// Hash the given password using the given salt and ops limits, placing the result in the given
/// buffer. Note that the length of "out" is mostly arbitrary.
pub fn derive_key(
    out: &mut [u8],
    password: &[u8],
    salt: &Salt,
    ops_limit: usize,
    mem_limit: usize,
) -> Result<()> {
    let result = pwhash::derive_key(
        out,
        password,
        &salt.0,
        pwhash::OpsLimit(ops_limit),
        pwhash::MemLimit(mem_limit),
    );
    if result.is_err() {
        // NOTE: We handle this error gracefully, but in reality (by inspecting the
        // libsodium source code) the only way this can actually fail is if the input
        // password is *enormous*. So, this won't really fail in practice.
        return Err(Error::Internal(format!(
            "deriving key from password failed"
        )));
    }

    Ok(())
}
