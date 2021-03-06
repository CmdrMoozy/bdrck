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
use rmp_serde;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::hash;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::randombytes::randombytes;
use std::fmt;

/// This module uses xsalsa20poly1305, whose nonces are 24 bytes long.
pub const NONCE_BYTES: usize = secretbox::NONCEBYTES;
/// This module uses sha512, which produces 64 byte digests.
pub const DIGEST_BYTES: usize = hash::DIGESTBYTES;
/// scryptsalsa208sha256 uses 32 byte salts.
pub const SALT_BYTES: usize = pwhash::SALTBYTES;
/// xsalsa20poly1305 uses 32 byte keys.
pub const KEY_BYTES: usize = secretbox::KEYBYTES;

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

/// A cryptographic nonce is an arbitrary number that can be used only once
/// (e.g. for encryption).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Nonce {
    nonce: secretbox::Nonce,
}

impl Default for Nonce {
    /// Return a new randomly generated nonce. This is the default, because it's
    /// safer / less error prone vs. e.g. accidentally using a zeroed out nonce
    /// twice.
    fn default() -> Self {
        Nonce {
            nonce: secretbox::gen_nonce(),
        }
    }
}

impl Nonce {
    /// Return a new, zero-initialized nonce.
    pub fn new() -> Self {
        Nonce {
            nonce: secretbox::Nonce([0; NONCE_BYTES]),
        }
    }

    /// Construct a new Nonce from raw bytes. The given byte slice must be
    /// exactly NONCE_BYTES long.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != NONCE_BYTES {
            return Err(Error::InvalidArgument(format!(
                "expected {} Nonce bytes, got {}",
                NONCE_BYTES,
                bytes.len()
            )));
        }

        let mut nonce = secretbox::Nonce([0; NONCE_BYTES]);
        for (dst, src) in nonce.0.iter_mut().zip(bytes.iter()) {
            *dst = *src;
        }

        Ok(Nonce { nonce: nonce })
    }

    /// Increment this nonce's bytes by 1. This is useful for counter-style
    /// nonces.
    pub fn increment(self) -> Self {
        let mut nonce = self.nonce;
        nonce.increment_le_inplace();
        Nonce { nonce: nonce }
    }

    /// Access the raw bytes which make up this Nonce.
    pub fn as_bytes(&self) -> &[u8] {
        &self.nonce.0
    }
}

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

/// An AbstractKey is any cryptographic structure which supports encryption and
/// decryption.
pub trait AbstractKey {
    /// The Error type this key's functions can return.
    type Error: std::error::Error;

    /// Return a digest/signature computed from this key.
    fn get_digest(&self) -> Digest;

    /// Encrypt the given plaintext with this key. This function optionally
    /// takes a nonce as an argument. If this key's encryption algorithm
    /// utilizes a nonce, the provided one will be used.
    ///
    /// This function returns the ciphertext, as well as a Nonce (if one was
    /// used for encryption). If a Nonce was provided, that same Nonce is
    /// returned.
    fn encrypt(
        &self,
        plaintext: &[u8],
        nonce: Option<Nonce>,
    ) -> std::result::Result<(Option<Nonce>, Vec<u8>), Self::Error>;

    /// Decrypt the given ciphertext using this key and the nonce which was
    /// generated at encryption time (if any), returning the plaintext.
    fn decrypt(
        &self,
        nonce: Option<&Nonce>,
        ciphertext: &[u8],
    ) -> std::result::Result<Vec<u8>, Self::Error>;
}

/// A WrappedPayload is the data which was wrapped by a key. Because keys can be
/// wrapped arbitrarily many times, the unwrapped payload may either be a real
/// key, or it may be another wrapped key.
#[derive(Clone, Deserialize, Serialize)]
pub enum WrappedPayload {
    /// The thing which has been wrapped is a Key, so unwrapping this payload
    /// will return the raw Key.
    Key(Key),
    /// The thing that has been wrapped is another wrapped payload, so more than
    /// one unwrap operation is needed to access the raw underlying Key.
    WrappedKey(WrappedKey),
}

/// A Wrappable is any object it is useful to "wrap" (encrypt) with a key.
pub trait Wrappable {
    /// This function "wraps" (encrypts, roughly) self with the given
    /// AbstractKey.
    fn wrap<K: AbstractKey>(self, key: &K) -> Result<WrappedKey>;
}

/// In this module's terminology, a Key is a cryptographic key of any type
/// *which is suitable to use for encryption* (i.e., is has not been wrapped).
#[derive(Clone, Deserialize, Serialize)]
pub struct Key {
    key: secretbox::Key,
}

impl AbstractKey for Key {
    type Error = Error;

    fn get_digest(&self) -> Digest {
        Digest::from_bytes(self.key.0.as_ref())
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        nonce: Option<Nonce>,
    ) -> std::result::Result<(Option<Nonce>, Vec<u8>), Self::Error> {
        let nonce = nonce.unwrap_or_else(|| Nonce::default());
        let ciphertext = secretbox::seal(plaintext, &nonce.nonce, &self.key);
        Ok((Some(nonce), ciphertext))
    }

    fn decrypt(
        &self,
        nonce: Option<&Nonce>,
        ciphertext: &[u8],
    ) -> std::result::Result<Vec<u8>, Self::Error> {
        let result = secretbox::open(
            ciphertext,
            match nonce {
                None => {
                    return Err(Error::InvalidArgument(format!(
                        "decrypting with a Key requires a nonce"
                    ))
                    .into());
                }
                Some(nonce) => &nonce.nonce,
            },
            &self.key,
        );
        if result.is_err() {
            return Err(
                Error::InvalidArgument(format!("failed to decrypt with incorrect Key")).into(),
            );
        }
        Ok(result.ok().unwrap())
    }
}

impl Wrappable for Key {
    /// Wrap this Wrappable type by encrypting it with the given key.
    fn wrap<K: AbstractKey>(self, key: &K) -> Result<WrappedKey> {
        let payload = WrappedPayload::Key(self);
        WrappedKey::wrap_payload(payload, key)
    }
}

impl Key {
    /// This is a utility used to implement our various public constructors.
    /// This constructor builds a new NormalKey from the given raw bytes.
    fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let key = secretbox::Key::from_slice(data.as_slice());
        if key.is_none() {
            return Err(Error::InvalidArgument(format!(
                "invalid Key; expected {} bytes, got {}",
                KEY_BYTES,
                data.len()
            )));
        }

        Ok(Key { key: key.unwrap() })
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
        let mut key_buffer = vec![0; KEY_BYTES];

        {
            let result = pwhash::derive_key(
                key_buffer.as_mut_slice(),
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
    nonce: Option<Nonce>,
    /// The digest of the key used to wrap this key.
    wrapping_digest: Digest,
}

impl Wrappable for WrappedKey {
    fn wrap<K: AbstractKey>(self, key: &K) -> Result<WrappedKey> {
        let payload = WrappedPayload::WrappedKey(self);
        WrappedKey::wrap_payload(payload, key)
    }
}

impl WrappedKey {
    fn wrap_payload<K: AbstractKey>(payload: WrappedPayload, key: &K) -> Result<Self> {
        let serialized = rmp_serde::to_vec(&payload)?;
        let (nonce, ciphertext) = match key.encrypt(serialized.as_slice(), None) {
            Err(e) => return Err(Error::Crypto(format!("wrapping key failed: {}", e))),
            Ok(tuple) => tuple,
        };
        let digest = key.get_digest();

        Ok(WrappedKey {
            data: ciphertext,
            nonce: nonce,
            wrapping_digest: digest,
        })
    }

    /// Return a digest/signature computed from this key.
    pub fn get_digest(&self) -> Digest {
        Digest::from_bytes(self.data.as_slice())
    }

    /// Return the digest/signature of the outermost key used to wrap this key.
    pub fn get_wrapping_digest(&self) -> &Digest {
        &self.wrapping_digest
    }

    /// Unwrap this WrappedKey using the given key for decryption. This can
    /// return either a Key, or another WrappedKey if the underlying key was
    /// wrapped more than one time.
    pub fn unwrap<K: AbstractKey>(self, key: &K) -> Result<WrappedPayload> {
        if key.get_digest() != self.wrapping_digest {
            return Err(Error::InvalidArgument(format!(
                "the specified key is not the correct wrapping key"
            )));
        }
        let plaintext = match key.decrypt(self.nonce.as_ref(), self.data.as_slice()) {
            Err(e) => return Err(Error::Crypto(format!("unwrapping key failed: {}", e))),
            Ok(pt) => pt,
        };
        Ok(rmp_serde::from_slice(plaintext.as_slice())?)
    }
}
