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

use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::hash;
use sodiumoxide::crypto::secretbox;
use std::fmt;

/// This module uses xsalsa20poly1305, whose nonces are 24 bytes long.
pub const NONCE_BYTES: usize = secretbox::NONCEBYTES;
/// This module uses sha512, which produces 64 byte digests.
pub const DIGEST_BYTES: usize = hash::DIGESTBYTES;
/// xsalsa20poly1305 uses 32 byte keys.
pub const KEY_BYTES: usize = secretbox::KEYBYTES;

/// A cryptographic nonce is an arbitrary number that can be used only once
/// (e.g. for encryption).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Nonce([u8; NONCE_BYTES]);

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

/// In this module's terminology, a Key is a cryptographic key of any type
/// *which is suitable to use for encryption* (i.e., is has not been wrapped).
pub struct Key([u8; KEY_BYTES]);

/// A wrapped key is a Key which has been wrapped (encrypted) with another key.
/// This is useful because it lets us have e.g. a single "master key" which is
/// wrapped by several sub-keys, which can be added / removed at will without
/// having to actually re-encrypt all of the data encrypted with the "master
/// key".
#[derive(Clone, Deserialize, Serialize)]
pub struct WrappedKey {
    /// The raw wrapped bytes. This data needs to be unwrapped (decrypted)
    /// before it can be used.
    data: Vec<u8>,
    /// The nonce used to encrypt this wrapped key, if applicable.
    nonce: Option<secretbox::Nonce>,
    /// The signature of the key used to wrap the underlying key.
    signature: hash::Digest,
}
