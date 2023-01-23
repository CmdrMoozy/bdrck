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

use crate::crypto::digest::Digest;
use crate::crypto::key::{AbstractKey, Nonce};
use crate::error::*;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// A wrapped key is an `AbstractKey` which has been wrapped (encrypted) with another `AbstractKey`.
/// This is useful because it lets us have e.g. a single "master key" which is wrapped by several
/// sub-keys, which can be added / removed at will without having to actually re-encrypt all of the
/// data encrypted with the "master key".
#[derive(Deserialize, Serialize)]
pub struct WrappedKey {
    /// The `serialize`-ed `AbstractKey` data, encrypted. This data has to be unwrapped (decrypted)
    /// before it can be used.
    data: Vec<u8>,
    /// The nonce used to encrypt, if any.
    nonce: Option<Nonce>,
    /// The digest of the key used to wrap this key.
    wrapping_digest: Digest,
}

impl WrappedKey {
    /// Wrap the key `to_wrap` with the key `wrap_with` used for encryption.
    pub fn wrap<KA: AbstractKey, KB: AbstractKey>(to_wrap: &KA, wrap_with: &KB) -> Result<Self> {
        let data = match to_wrap.serialize() {
            Err(e) => return Err(Error::Crypto(format!("serializing key failed: {}", e))),
            Ok(d) => d,
        };

        let (nonce, data) = match wrap_with.encrypt(&data, None) {
            Err(e) => return Err(Error::Crypto(format!("wrapping key failed: {}", e))),
            Ok(nd) => nd,
        };

        Ok(WrappedKey {
            data: data,
            nonce: nonce,
            wrapping_digest: wrap_with.get_digest(),
        })
    }

    /// Unwrap the previously wrapped key this structure represents. This basically decrypts and
    /// then deserializes the underlying key data, returning the newly constructed key.
    pub fn unwrap<KA: AbstractKey, KB: AbstractKey>(&self, wrapped_with: &KB) -> Result<KA> {
        debug!(
            "trying to unwrap key {:?} with wrapping key {:?}, expected wrapping digest {:?}",
            self.get_digest(),
            wrapped_with.get_digest(),
            self.wrapping_digest
        );
        if wrapped_with.get_digest() != self.wrapping_digest {
            return Err(Error::InvalidArgument(format!(
                "the specified key is not the correct wrapping key"
            )));
        }

        let data = match wrapped_with.decrypt(self.nonce.as_ref(), self.data.as_slice()) {
            Err(e) => return Err(Error::Crypto(format!("unwrapping key failed: {}", e))),
            Ok(d) => d,
        };

        match KA::deserialize(data) {
            Err(e) => return Err(Error::Crypto(format!("deserializing key failed: {}", e))),
            Ok(k) => Ok(k),
        }
    }

    /// Return a digest/signature computed from the encrypted key data.
    pub fn get_digest(&self) -> Digest {
        Digest::from_bytes(self.data.as_slice())
    }

    /// Return the digest/signature of the outermost key used to wrap this key.
    pub fn get_wrapping_digest(&self) -> &Digest {
        &self.wrapping_digest
    }
}
