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

use crypto::key::{AbstractKey, Key, Nonce, Wrappable, WrappedKey, WrappedPayload};
use error::*;
use msgpack;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

lazy_static! {
    /// This token is used to verify that authentication was successful. We encrypt it with a master
    /// key which we then wrap with user key(s), so we can verify that the user presented a valid
    /// key by trying to decrypt this token.
    static ref AUTH_TOKEN_CONTENTS: Vec<u8> = "3c017f717b39247c351154a41d2850e4187284da4b928f13c723d54440ba2dfe".bytes().collect();
}

/// KeyStoreContents is an implementation detail of KeyStore, which encapsulates
/// the portion of the KeyStore's data which is actually persisted to disk.
#[derive(Deserialize, Serialize)]
struct KeyStoreContents {
    pub token_nonce: Option<Nonce>,
    pub token: Vec<u8>,
    pub wrapped_keys: Vec<WrappedKey>,
}

impl KeyStoreContents {
    /// Constrct a new KeyStoreContents from scratch, using the given master
    /// key.
    pub fn new(master_key: &mut Key) -> Result<KeyStoreContents> {
        let (nonce, ciphertext) = master_key.encrypt(AUTH_TOKEN_CONTENTS.as_slice())?;
        Ok(KeyStoreContents {
            token_nonce: nonce,
            token: ciphertext,
            wrapped_keys: Vec::new(),
        })
    }

    /// Deserialize this structure from its binary serialized format in the
    /// given file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<KeyStoreContents> {
        let file = File::open(path)?;
        Ok(msgpack::from_read(file)?)
    }

    /// Write this KeyStoreContents as a binary serialized structure to a file
    /// at the given path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = msgpack::to_vec(self)?;
        let mut file = File::create(path)?;
        Ok(file.write_all(data.as_slice())?)
    }

    /// Returns true if the given key is this structure's "master key" which was
    /// used to encrypt the "token" upon construction.
    pub fn is_master_key(&self, key: &mut Key) -> bool {
        let decrypted = match key.decrypt(self.token_nonce.as_ref(), self.token.as_slice()) {
            Err(_) => return false,
            Ok(d) => d,
        };
        decrypted.as_slice() == AUTH_TOKEN_CONTENTS.as_slice()
    }

    /// Add the given wrapped key to this KeyStoreContents. No real validation
    /// is performed on this wrapped key, other than to check if it already
    /// exists in this structure (in which case false is returned).
    pub fn add_key(&mut self, wrapped_key: WrappedKey) -> bool {
        if self.wrapped_keys
            .iter()
            .filter(|k| k.get_wrapping_digest() == wrapped_key.get_wrapping_digest())
            .count() > 0
        {
            return false;
        }
        self.wrapped_keys.push(wrapped_key);
        true
    }

    /// Remove the wrapped key which was wrapped using the given wrapping key
    /// from this KeyStoreContents. True/false is returned to indicate whether
    /// a matching key was actually found. It is an error to remove the last
    /// wrapped key from this structure.
    pub fn remove_key<K: AbstractKey>(&mut self, wrapping_key: &K) -> Result<bool> {
        let original_length = self.wrapped_keys.len();
        let wrapped_keys: Vec<WrappedKey> = self.wrapped_keys
            .iter()
            .filter(|k| *k.get_wrapping_digest() != wrapping_key.get_digest())
            .cloned()
            .collect();
        if wrapped_keys.is_empty() {
            bail!("Refusing to remove all valid keys from this KeyStore");
        }
        self.wrapped_keys = wrapped_keys;
        Ok(self.wrapped_keys.len() != original_length)
    }
}

/// A KeyStore is a structure which contains a single "master key", wrapped with
/// one or more other keys. This is useful in cases where we want to encrypt
/// data with a single key, while allowing users to add or remove keys at will,
/// without having to a) re-encrypt the data when the keys change, or b) store
/// multiple copies of the plaintext encrypted with the various different keys.
///
/// For example, users may want to be able to access a resource with *either* a
/// password or a hardware authentication key, and the data they want to encrypt
/// is relatively large (so re-encryption is expensive).
///
/// A KeyStore essentially contains a set of one or more wrapped keys, which it
/// automatically loads from / persists to disk.
pub struct KeyStore {
    path: PathBuf,
    master_key: Key,
    contents: KeyStoreContents,
}

impl KeyStore {
    /// Construct a new KeyStore, which will be persisted to the given path. If
    /// a file already exists at the given path, it will be overwritten.
    fn new<P: AsRef<Path>>(path: P) -> Result<KeyStore> {
        let mut master_key = Key::new_random()?;
        let contents = KeyStoreContents::new(&mut master_key)?;

        Ok(KeyStore {
            path: path.as_ref().to_path_buf(),
            master_key: master_key,
            contents: contents,
        })
    }

    /// Open an existing KeyStore which was previously persisted to the given
    /// path. If the given path does not contain a valid KeyStore, or if the
    /// KeyStore at the given path can't be "unwrapped" with the given key, an
    /// error is returned instead.
    fn open<P: AsRef<Path>, K: AbstractKey>(path: P, key: &mut K) -> Result<KeyStore> {
        let contents = KeyStoreContents::open(path.as_ref())?;
        let mut master_key: Option<Key> = None;
        for wrapped_key in contents.wrapped_keys.iter() {
            if let Ok(payload) = wrapped_key.clone().unwrap(key) {
                let mut unwrapped_key = match payload {
                    WrappedPayload::Key(k) => k,
                    _ => continue,
                };
                if contents.is_master_key(&mut unwrapped_key) {
                    master_key = Some(unwrapped_key);
                    break;
                }
            }
        }

        if master_key.is_none() {
            bail!("Failed to unwrap master key with the provided wrapping key");
        }

        Ok(KeyStore {
            path: path.as_ref().to_path_buf(),
            master_key: master_key.unwrap(),
            contents: contents,
        })
    }

    /// Open an existing KeyStore if the given path exists, or create a brand
    /// new KeyStore otherwise and add the given key to it.
    pub fn open_or_new<P: AsRef<Path>, K: AbstractKey>(path: P, key: &mut K) -> Result<KeyStore> {
        if path.as_ref().exists() {
            Self::open(path, key)
        } else {
            let mut keystore = Self::new(path)?;
            keystore.add_key(key)?;
            Ok(keystore)
        }
    }

    /// Return the unwrapped master key from this KeyStore.
    pub fn get_master_key(&self) -> &Key {
        &self.master_key
    }

    /// Return the mutable unwrapped master key from this KeyStore.
    pub fn get_master_key_mut(&mut self) -> &mut Key {
        &mut self.master_key
    }

    /// Add the given wrapping key to this KeyStore. On future
    /// open_or_new calls, this new key can be used to open the KeStore. Return
    /// whether the key was successfully added (true), or if it was already
    /// present in the KeyStore (false).
    pub fn add_key<K: AbstractKey>(&mut self, key: &mut K) -> Result<bool> {
        Ok(self.contents.add_key(self.master_key.clone().wrap(key)?))
    }

    /// Remove the given key from this KeyStore, so it can no longer be used to
    /// unwrap / open the KeyStore. Returns true if the key was removed, or
    /// false if the given key wasn't found in this KeyStore. It is an error to
    /// remove the last wrapping key from a KeyStore (doing so would leave it
    /// unopenable in the future).
    pub fn remove_key<K: AbstractKey>(&mut self, key: &K) -> Result<bool> {
        self.contents.remove_key(key)
    }
}

impl Drop for KeyStore {
    fn drop(&mut self) {
        self.contents.save(self.path.as_path()).unwrap();
    }
}
