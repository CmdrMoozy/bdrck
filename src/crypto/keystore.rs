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
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

lazy_static! {
    /// This token is used to verify that authentication was successful. We encrypt it with a master
    /// key which we then wrap with user key(s), so we can verify that the user presented a valid
    /// key by trying to decrypt this token.
    static ref AUTH_TOKEN_CONTENTS: Vec<u8> = "3c017f717b39247c351154a41d2850e4187284da4b928f13c723d54440ba2dfe".bytes().collect();
}

/// KeyStoreContents is an implementation detail of KeyStore, which encapsulates
/// the portion of the KeyStore's data which is actually persisted.
#[derive(Deserialize, Serialize)]
struct KeyStoreContents {
    token_nonce: Option<Nonce>,
    token: Vec<u8>,
    wrapped_keys: Vec<WrappedKey>,
}

impl KeyStoreContents {
    /// Constrct a new KeyStoreContents from scratch, using the given master
    /// key.
    fn new(master_key: &Key) -> Result<Self> {
        let (nonce, ciphertext) = master_key.encrypt(AUTH_TOKEN_CONTENTS.as_slice(), None)?;
        Ok(KeyStoreContents {
            token_nonce: nonce,
            token: ciphertext,
            wrapped_keys: Vec::new(),
        })
    }

    /// Deserialize a KeyStoreContents from the given bytes.
    fn from_slice(data: &[u8]) -> Result<Self> {
        Ok(msgpack::from_slice(data)?)
    }

    /// Deserialize a KeyStoreContents from the given reader.
    fn from_read<R: Read>(rd: R) -> Result<Self> {
        Ok(msgpack::from_read(rd)?)
    }

    /// Serialize this KeyStoreContents to a byte vector, which can then be
    /// persisted in e.g. a file or whatever.
    fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(msgpack::to_vec(self)?)
    }

    /// Returns true if the given key is this structure's "master key" which was
    /// used to encrypt the "token" upon construction.
    fn is_master_key(&self, key: &Key) -> bool {
        let decrypted = match key.decrypt(self.token_nonce.as_ref(), self.token.as_slice()) {
            Err(_) => return false,
            Ok(d) => d,
        };
        decrypted.as_slice() == AUTH_TOKEN_CONTENTS.as_slice()
    }

    /// Add the given wrapped key to this KeyStoreContents. No real validation
    /// is performed on this wrapped key, other than to check if it already
    /// exists in this structure (in which case false is returned).
    fn add_key(&mut self, wrapped_key: WrappedKey) -> bool {
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
    fn remove_key<K: AbstractKey>(&mut self, wrapping_key: &K) -> Result<bool> {
        let original_length = self.wrapped_keys.len();
        let wrapped_keys: Vec<WrappedKey> = self.wrapped_keys
            .iter()
            .filter(|k| *k.get_wrapping_digest() != wrapping_key.get_digest())
            .cloned()
            .collect();
        if wrapped_keys.is_empty() {
            return Err(Error::Precondition(format_err!(
                "Refusing to remove all valid keys from this KeyStore"
            )));
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
    master_key: Key,
    contents: KeyStoreContents,
}

impl KeyStore {
    /// Construct a new KeyStore. A new master key is generated from scratch,
    /// and is wrapped with the given key (in other words, the given key is
    /// added to the KeyStore).
    pub fn new<K: AbstractKey>(key: &K) -> Result<Self> {
        let master_key = Key::new_random()?;
        let contents = KeyStoreContents::new(&master_key)?;

        let mut store = KeyStore {
            master_key: master_key,
            contents: contents,
        };

        store.add_key(key)?;
        Ok(store)
    }

    fn open<K: AbstractKey>(contents: KeyStoreContents, key: &K) -> Result<Self> {
        let mut master_key: Option<Key> = None;
        for wrapped_key in contents.wrapped_keys.iter() {
            if let Ok(payload) = wrapped_key.clone().unwrap(key) {
                let unwrapped_key = match payload {
                    WrappedPayload::Key(k) => k,
                    _ => continue,
                };
                if contents.is_master_key(&unwrapped_key) {
                    master_key = Some(unwrapped_key);
                    break;
                }
            }
        }

        if master_key.is_none() {
            return Err(Error::InvalidArgument(format_err!(
                "KeyStore unlocking failed: the given key is not present in this KeyStore"
            )));
        }

        Ok(KeyStore {
            master_key: master_key.unwrap(),
            contents: contents,
        })
    }

    /// Open the KeyStore (attempt to unwrap the master key) by deserializing
    /// the given KeyStore bytes.
    pub fn open_slice<K: AbstractKey>(data: &[u8], key: &K) -> Result<Self> {
        Self::open(KeyStoreContents::from_slice(data)?, key)
    }

    /// Open the KeyStore (attempt to unwrap the master key) by deserializing
    /// the KeyStore bytes read from the given reader.
    pub fn open_read<R: Read, K: AbstractKey>(rd: R, key: &K) -> Result<Self> {
        Self::open(KeyStoreContents::from_read(rd)?, key)
    }

    /// Serialize this KeyStore, so it can be persisted and then reloaded later.
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        self.contents.to_vec()
    }

    /// Return the unwrapped master key from this KeyStore.
    pub fn get_master_key(&self) -> &Key {
        &self.master_key
    }

    /// Add the given wrapping key to this KeyStore. On future
    /// open_or_new calls, this new key can be used to open the KeStore. Return
    /// whether the key was successfully added (true), or if it was already
    /// present in the KeyStore (false).
    pub fn add_key<K: AbstractKey>(&mut self, key: &K) -> Result<bool> {
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

/// DiskKeyStore is a very simple wrapper around KeyStore, which deals with
/// persisting it to disk. This is provided because it is expected this is a
/// very common use case, but users of this library can just use KeyStore
/// directly and persist it however they like.
pub struct DiskKeyStore {
    path: PathBuf,
    inner: KeyStore,
}

impl DiskKeyStore {
    /// Construct a new DiskKeyStore, which will be persisted to the given path.
    /// If a file already exists at the given path, it will be overwritten.
    pub fn new<P: AsRef<Path>, K: AbstractKey>(path: P, key: &K) -> Result<Self> {
        Ok(DiskKeyStore {
            path: path.as_ref().to_path_buf(),
            inner: KeyStore::new(key)?,
        })
    }

    /// Open an existing DiskKeyStore which was previously persisted to the
    /// given path. If the given path does not contain a valid DiskKeyStore, or
    /// if the DiskKeyStore at the given path can't be "unwrapped" with the
    /// given key, an error is returned instead.
    pub fn open<P: AsRef<Path>, K: AbstractKey>(path: P, key: &K) -> Result<Self> {
        let mut f = File::open(path.as_ref())?;
        Ok(DiskKeyStore {
            path: path.as_ref().to_path_buf(),
            inner: KeyStore::open_read(&mut f, key)?,
        })
    }

    /// Open an existing KeyStore if the given path exists, or create a brand
    /// new KeyStore otherwise and add the given key to it.
    pub fn open_or_new<P: AsRef<Path>, K: AbstractKey>(path: P, key: &K) -> Result<Self> {
        if path.as_ref().exists() {
            Self::open(path, key)
        } else {
            Self::new(path, key)
        }
    }
}

impl Deref for DiskKeyStore {
    type Target = KeyStore;

    fn deref(&self) -> &KeyStore {
        &self.inner
    }
}

impl DerefMut for DiskKeyStore {
    fn deref_mut(&mut self) -> &mut KeyStore {
        &mut self.inner
    }
}

impl Drop for DiskKeyStore {
    fn drop(&mut self) {
        let mut f = File::create(self.path.as_path()).unwrap();
        let data = self.inner.to_vec().unwrap();
        f.write_all(data.as_slice()).unwrap();
    }
}
