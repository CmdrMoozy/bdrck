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

use error::*;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::hash;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::randombytes::randombytes;
use std::collections::VecDeque;
use std::fmt;

/// This module uses xsalsa20poly1305, whose nonces are 24 bytes long.
pub const NONCE_BYTES: usize = secretbox::NONCEBYTES;
/// This module uses sha512, which produces 64 byte digests.
pub const DIGEST_BYTES: usize = hash::DIGESTBYTES;
/// scryptsalsa208sha256 uses 32 byte salts.
pub const SALT_BYTES: usize = pwhash::SALTBYTES;
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

impl Digest {
    pub fn from_bytes(data: &[u8]) -> Self {
        Digest(hash::hash(data).0)
    }
}

/// A salt is an arbitrary byte sequence which is used for password-based key
/// derivation.
pub struct Salt([u8; SALT_BYTES]);

pub trait AbstractKey {
    /// Return a digest/signature computed from this key.
    fn get_digest(&self) -> Digest;
}

/// In this module's terminology, a Key is a cryptographic key of any type
/// *which is suitable to use for encryption* (i.e., is has not been wrapped).
pub struct Key([u8; KEY_BYTES]);

impl AbstractKey for Key {
    fn get_digest(&self) -> Digest {
        Digest::from_bytes(self.0.as_ref())
    }
}

impl Key {
    /// This is a utility used to implement our various public constructors.
    /// This constructor builds a new NormalKey from the given raw bytes.
    fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let key = secretbox::Key::from_slice(data.as_slice());
        if key.is_none() {
            bail!(
                "Invalid key; expected {} bytes, got {}",
                KEY_BYTES,
                data.len()
            );
        }
        Ok(Key(key.unwrap().0))
    }

    /// Generate a new random key.
    pub fn new_random() -> Result<Self> {
        Self::from_bytes(randombytes(KEY_BYTES))
    }

    /// Derive a new key from the given password. Note that the derived key will
    /// be different if any of the parameters to this function change, so they
    /// need to remain fixed if you e.g. re-derive the key to decrypt some
    /// previously-encrypted data.
    pub fn new_password(
        password: &[u8],
        salt: &Salt,
        ops_limit: usize,
        mem_limit: usize,
    ) -> Result<Self> {
        let salt = pwhash::Salt::from_slice(salt.0.as_ref()).unwrap();
        let mut key_buffer = vec![0; KEY_BYTES];

        {
            let result = pwhash::derive_key(
                key_buffer.as_mut_slice(),
                password,
                &salt,
                pwhash::OpsLimit(ops_limit),
                pwhash::MemLimit(mem_limit),
            );
            if result.is_err() {
                // NOTE: We handle this error gracefully, but in reality (by inspecting the
                // libsodium source code) the only way this can actually fail is if the input
                // password is *enormous*. So, this won't really fail in practice.
                bail!("Deriving key from password failed");
            }
        }

        Self::from_bytes(key_buffer)
    }
}

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
    /// The digests of the keys used to wrap the underlying Key. When this key
    /// is unwrapped, the *last* digest is popped. If this key is wrapped yet
    /// again, the new wrapping key's digest is pushed onto the *back* of this
    /// deque.
    wrapping_digests: VecDeque<Digest>,
}

impl WrappedKey {
    /// Return a digest/signature computed from this key.
    pub fn get_digest(&self) -> Digest {
        Digest::from_bytes(self.data.as_slice())
    }

    /// Return the digest/signature of the outermost key used to wrap this key.
    pub fn get_wrapping_digest(&self) -> &Digest {
        self.wrapping_digests.back().unwrap()
    }
}
