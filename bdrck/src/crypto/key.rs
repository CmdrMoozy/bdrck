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

use crate::crypto::compat::{self, Compatible};
use crate::crypto::digest::{derive_key, Digest, Salt};
use crate::crypto::secret::Secret;
use crate::crypto::util::*;
use crate::error::*;
use halite_sys;
use libc::c_ulonglong;
use serde::{Deserialize, Serialize};

/// This module uses xsalsa20poly1305, whose nonces are 24 bytes long.
pub const NONCE_BYTES: usize = halite_sys::crypto_secretbox_xsalsa20poly1305_NONCEBYTES as usize;
/// xsalsa20poly1305 uses 32 byte keys.
pub const KEY_BYTES: usize = halite_sys::crypto_secretbox_xsalsa20poly1305_KEYBYTES as usize;
/// xsalsa20poly1305 authenticator tags are 16 bytes.
pub const TAG_BYTES: usize = halite_sys::crypto_secretbox_xsalsa20poly1305_MACBYTES as usize;

/// A cryptographic nonce is an arbitrary number that can be used only once
/// (e.g. for encryption).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Nonce {
    // NOTE: This is a proper structure instead of a simple tuple structure, because this way of
    // defining it is part of our serialization format. Changing it would cause us to be unable to
    // deserialize instances serialized based on an old version of the code.
    nonce: compat::Nonce,
}

impl Default for Nonce {
    /// Return a new randomly generated nonce. This is the default, because it's
    /// safer / less error prone vs. e.g. accidentally using a zeroed out nonce
    /// twice.
    fn default() -> Self {
        let mut nonce = Nonce::new();
        randombytes_into(&mut nonce.nonce.0);
        nonce
    }
}

impl Nonce {
    /// Return a new, zero-initialized nonce.
    pub fn new() -> Self {
        Nonce {
            nonce: compat::Nonce([0; NONCE_BYTES]),
        }
    }

    /// Construct a Nonce from a properly sized byte slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        Ok(Nonce {
            nonce: compat::Nonce::from_slice(bytes)?,
        })
    }

    /// Increment this nonce's bytes by 1. This is useful for counter-style
    /// nonces.
    pub fn increment(mut self) -> Self {
        debug_assert!(crate::init_done());
        unsafe {
            halite_sys::sodium_increment(self.nonce.0.as_mut_ptr(), NONCE_BYTES as u64);
        }
        self
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

    /// Encrypt the given plaintext with this key. This function optionally
    /// takes a nonce as an argument. If this key's encryption algorithm
    /// utilizes a nonce, the provided one will be used.
    ///
    /// This function returns the ciphertext, as well as a Nonce (if one was
    /// used for encryption). If a Nonce was provided, that same Nonce is
    /// returned.
    fn encrypt(
        &self,
        plaintext: &Secret,
        nonce: Option<Nonce>,
    ) -> std::result::Result<(Option<Nonce>, Vec<u8>), Self::Error>;

    /// Decrypt the given ciphertext using this key and the nonce which was
    /// generated at encryption time (if any), returning the plaintext.
    fn decrypt(
        &self,
        nonce: Option<&Nonce>,
        ciphertext: &[u8],
    ) -> std::result::Result<Secret, Self::Error>;
}

/// In this module's terminology, a Key is a cryptographic key of any type
/// *which is suitable to use for encryption* (i.e., is has not been wrapped).
pub struct Key {
    key_data: Secret,
}

impl AbstractKey for Key {
    type Error = Error;

    fn get_digest(&self) -> Digest {
        Digest::from_secret(&self.key_data)
    }

    fn serialize(&self) -> std::result::Result<Secret, Self::Error> {
        self.key_data.try_clone()
    }

    fn deserialize(data: Secret) -> std::result::Result<Self, Self::Error> {
        if data.len() != KEY_BYTES {
            return Err(Error::InvalidArgument(format!(
                "invalid Key data; expected {} bytes, found {}",
                KEY_BYTES,
                data.len()
            )));
        }

        Ok(Key { key_data: data })
    }

    fn encrypt(
        &self,
        plaintext: &Secret,
        nonce: Option<Nonce>,
    ) -> std::result::Result<(Option<Nonce>, Vec<u8>), Self::Error> {
        let nonce = nonce.unwrap_or_else(Nonce::default);

        let buf = plaintext.try_clone()?;
        let mut tag = [0; TAG_BYTES];
        debug_assert!(crate::init_done());
        unsafe {
            halite_sys::crypto_secretbox_detached(
                buf.slice_ptr(),
                tag.as_mut_ptr(),
                buf.slice_ptr(),
                buf.len() as c_ulonglong,
                nonce.nonce.0.as_ptr(),
                self.key_data.slice_ptr(),
            );
        }

        let mut ret = Vec::new();
        ret.extend_from_slice(&tag);
        ret.extend_from_slice(unsafe { buf.as_slice() });

        Ok((Some(nonce), ret))
    }

    fn decrypt(
        &self,
        nonce: Option<&Nonce>,
        ciphertext: &[u8],
    ) -> std::result::Result<Secret, Self::Error> {
        if ciphertext.len() < TAG_BYTES {
            return Err(Error::InvalidArgument(format!(
                "can't decrypt ciphertext which is missing an authentication tag"
            ))
            .into());
        }

        let nonce = match nonce {
            None => {
                return Err(Error::InvalidArgument(format!(
                    "decrypting with a Key requires a Nonce"
                ))
                .into())
            }
            Some(n) => n,
        };

        let (tag, ciphertext) = ciphertext.split_at(TAG_BYTES);

        let mut plaintext = Secret::with_len(ciphertext.len())?;
        unsafe { plaintext.as_mut_slice() }.copy_from_slice(ciphertext);

        debug_assert!(crate::init_done());
        if unsafe {
            halite_sys::crypto_secretbox_open_detached(
                plaintext.slice_ptr(),
                plaintext.slice_ptr(),
                tag.as_ptr(),
                plaintext.len() as c_ulonglong,
                nonce.nonce.0.as_ptr(),
                self.key_data.slice_ptr(),
            )
        } == 0
        {
            Ok(plaintext)
        } else {
            Err(Error::InvalidArgument(format!("failed to decrypt with incorrect Key")).into())
        }
    }
}

impl Key {
    /// Generate a new random key.
    pub fn new_random() -> Result<Self> {
        let mut key_buffer = Secret::with_len(KEY_BYTES)?;
        randombytes_into_secret(&mut key_buffer);
        Self::deserialize(key_buffer)
    }

    /// Derive a new key from the given password. Note that the derived key will
    /// be different if any of the parameters to this function change, so they
    /// need to remain fixed if you e.g. re-derive the key to decrypt some
    /// previously-encrypted data.
    pub fn new_password(
        password: &Secret,
        salt: &Salt,
        ops_limit: usize,
        mem_limit: usize,
    ) -> Result<Self> {
        let mut key_buffer = Secret::with_len(KEY_BYTES)?;
        derive_key(&mut key_buffer, password, salt, ops_limit, mem_limit)?;
        Self::deserialize(key_buffer)
    }
}
