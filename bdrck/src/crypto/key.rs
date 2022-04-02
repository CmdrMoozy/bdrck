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

use crate::crypto::digest::{derive_key, Digest, Salt};
use crate::crypto::secret::Secret;
use crate::error::*;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;
use sodiumoxide::randombytes::randombytes_into;

/// This module uses xsalsa20poly1305, whose nonces are 24 bytes long.
pub const NONCE_BYTES: usize = secretbox::NONCEBYTES;
/// xsalsa20poly1305 uses 32 byte keys.
pub const KEY_BYTES: usize = secretbox::KEYBYTES;

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

/// An AbstractKey is any cryptographic structure which supports encryption and
/// decryption.
pub trait AbstractKey: Sized {
    /// The Error type this key's functions can return.
    type Error: std::error::Error;

    /// Return a digest/signature computed from this key.
    fn get_digest(&self) -> Digest;

    /// Serialize this key out as a set of raw bytes.
    fn serialize(&self) -> std::result::Result<Secret, Self::Error>;

    /// Construct an instance of this key from a previously serialized instance of the key. In
    /// general, `deserialize` should accept data previously produced by this key's `serialize`
    /// implementation.
    fn deserialize(data: Secret) -> std::result::Result<Self, Self::Error>;

    // TODO: Refactor encrypt and decrypt to use Secret properly.

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

const fn key_data_len() -> usize {
    std::mem::size_of::<secretbox::Key>()
}

/// In this module's terminology, a Key is a cryptographic key of any type
/// *which is suitable to use for encryption* (i.e., is has not been wrapped).
pub struct Key {
    key_data: Secret,
}

impl AbstractKey for Key {
    type Error = Error;

    fn get_digest(&self) -> Digest {
        Digest::from_bytes(self.inner().0.as_ref())
    }

    fn serialize(&self) -> std::result::Result<Secret, Self::Error> {
        self.key_data.try_clone()
    }

    fn deserialize(data: Secret) -> std::result::Result<Self, Self::Error> {
        if data.len() != key_data_len() {
            return Err(Error::InvalidArgument(format!(
                "invalid Key data; expected {} bytes, found {}",
                key_data_len(),
                data.len()
            )));
        }

        Ok(Key { key_data: data })
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        nonce: Option<Nonce>,
    ) -> std::result::Result<(Option<Nonce>, Vec<u8>), Self::Error> {
        let nonce = nonce.unwrap_or_else(|| Nonce::default());
        let ciphertext = secretbox::seal(plaintext, &nonce.nonce, self.inner());
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
            self.inner(),
        );
        if result.is_err() {
            return Err(
                Error::InvalidArgument(format!("failed to decrypt with incorrect Key")).into(),
            );
        }
        Ok(result.ok().unwrap())
    }
}

impl Key {
    /// Generate a new random key.
    pub fn new_random() -> Result<Self> {
        let mut key_buffer = Secret::with_len(KEY_BYTES)?;
        randombytes_into(unsafe { key_buffer.as_mut_slice() });
        Self::deserialize(key_buffer)
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
        let mut key_buffer = Secret::with_len(KEY_BYTES)?;
        // TODO: Refactor derive_key to use Secret properly.
        derive_key(
            unsafe { key_buffer.as_mut_slice() },
            password,
            salt,
            ops_limit,
            mem_limit,
        )?;
        Self::deserialize(key_buffer)
    }

    unsafe fn key_ptr(&self) -> *mut secretbox::Key {
        self.key_data.slice_ptr() as *mut secretbox::Key
    }

    fn inner(&self) -> &secretbox::Key {
        unsafe { self.key_ptr().as_ref().unwrap() }
    }
}
