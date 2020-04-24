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

use crate::crypto::key::{AbstractKey, Key, Nonce, Wrappable, WrappedKey, WrappedPayload};
use crate::error::*;
use data_encoding;
use lazy_static::lazy_static;
use log::error;
use rmp_serde;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

lazy_static! {
    /// This token is used to verify that authentication was successful. We encrypt it with a master
    /// key which we then wrap with user key(s), so we can verify that the user presented a valid
    /// key by trying to decrypt this token.
    static ref AUTH_TOKEN_CONTENTS: Vec<u8> = "3c017f717b39247c351154a41d2850e4187284da4b928f13c723d54440ba2dfe".bytes().collect();
}

/// Returns true if the given key is this structure's "master key" which was
/// used to encrypt the `token` upon construction.
fn is_master_key<K: AbstractKey>(key: &K, nonce: Option<&Nonce>, token: &[u8]) -> bool {
    let decrypted = match key.decrypt(nonce, token) {
        Err(_) => return false,
        Ok(d) => d,
    };
    decrypted.as_slice() == AUTH_TOKEN_CONTENTS.as_slice()
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
#[derive(Deserialize, Serialize)]
pub struct KeyStore {
    /// The master key is never persisted or loaded from serialized bytes.
    /// Instead, KeyStore constructors are in charge of retrieving the master
    /// key from `wrapped_keys`, and populating this field.
    ///
    /// In other words, it is an invariant of KeyStore that after its
    /// constructor has returned, this field will *never* be None.
    #[serde(skip_serializing, skip_deserializing)]
    master_key: Option<Key>,

    token_nonce: Option<Nonce>,
    token: Vec<u8>,
    wrapped_keys: Vec<WrappedKey>,
}

impl KeyStore {
    /// Construct a new KeyStore. A new master key is generated from scratch.
    ///
    /// Note that, before this KeyStore can be meaningfully persisted, you need
    /// to wrap the master key with at least one key, via `add_key`.
    pub fn new() -> Result<Self> {
        // Generate a new master key. We'll store this *wrapped with `key`*.
        let master_key = Key::new_random()?;
        // Encrypt the auth token with the master key. This is so we can decrypt
        // it later, and verify we get the right result, to guarantee we have
        // the right master key.
        let (nonce, ciphertext) = master_key.encrypt(AUTH_TOKEN_CONTENTS.as_slice(), None)?;

        Ok(KeyStore {
            master_key: Some(master_key),
            token_nonce: nonce,
            token: ciphertext,
            wrapped_keys: Vec::new(),
        })
    }

    /// Load a previously-serialized (with `to_vec`) KeyStore from a byte slice.
    pub fn load_slice(data: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(data)?)
    }

    /// Load a previously-serialized (with `to_vec`) KeyStore from a reader.
    pub fn load_read<R: Read>(rd: R) -> Result<Self> {
        Ok(rmp_serde::from_read(rd)?)
    }

    /// Return a string which "uniquely" identifies this KeyStore.
    ///
    /// (This is quote "unique" because `KeyStore`s with identical master keys
    /// may return the same string here.) This string is mainly useful for
    /// debugging / logging purposes.
    pub fn get_id(&self) -> String {
        data_encoding::HEXLOWER.encode(&self.token)
    }

    /// Return whether or not this KeyStore is open.
    pub fn is_open(&self) -> bool {
        self.master_key.is_some()
    }

    /// Return whether or not this KeyStore is meaningfully "persistable". In
    /// other words, this returns whether or not this KeyStore has at least one
    /// wrapping key.
    ///
    /// Because the master key is not persisted in plain text, if a KeyStore has
    /// no wrapping keys (yet), it is not useful to persist it, as it can never
    /// be opened again.
    pub fn is_persistable(&self) -> bool {
        !self.wrapped_keys.is_empty()
    }

    /// Open this KeyStore (attempt to unwrap the master key) using the given
    /// wrapping key. If this fails, the structure will still be in a valid
    /// state, so you could e.g. try again with a different wrapping key.
    pub fn open<K: AbstractKey>(&mut self, key: &K) -> Result<()> {
        if self.master_key.is_some() {
            // We're already opened, this will be a no-op.
            return Ok(());
        }

        let mut master_key: Option<Key> = None;
        for wrapped_key in self.wrapped_keys.iter() {
            if let Ok(payload) = wrapped_key.clone().unwrap(key) {
                let unwrapped_key = match payload {
                    WrappedPayload::Key(k) => k,
                    _ => continue,
                };
                if is_master_key(
                    &unwrapped_key,
                    self.token_nonce.as_ref(),
                    self.token.as_slice(),
                ) {
                    master_key = Some(unwrapped_key);
                    break;
                }
            }
        }

        if master_key.is_none() {
            return Err(Error::InvalidArgument(format!(
                "KeyStore unlocking failed: the given key is not present in this KeyStore"
            )));
        }

        self.master_key = master_key;
        Ok(())
    }

    /// Serialize this KeyStore, so it can be persisted and then reloaded later.
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(self)?)
    }

    /// Return the unwrapped master key from this KeyStore. If this KeyStore
    /// has no master key (it was neither newly generated nor unwrapped), this
    /// will return an error instead.
    pub fn get_master_key(&self) -> Result<&Key> {
        if let Some(k) = self.master_key.as_ref() {
            return Ok(k);
        }
        Err(Error::Precondition(format!(
            "KeyStore must be opened before you can access the master key"
        )))
    }

    /// Add the given wrapping key to this KeyStore. When the KeyStore is opened
    /// in the future, this key can be used. Returns true if the key was
    /// successfully added, or false if it was already present in the KeyStore.
    ///
    /// If this KeyStore has no master key (it was neither newly generated nor
    /// unwrapped), this will return an error instead.
    pub fn add_key<K: AbstractKey>(&mut self, key: &K) -> Result<bool> {
        let wrapped_key = match self.master_key.clone() {
            None => {
                return Err(Error::Precondition(format!(
                    "KeyStore must be `new` or opened to add keys"
                )))
            }
            Some(k) => k,
        }
        .wrap(key)?;

        // If this key is already in the KeyStore, just return.
        if self
            .wrapped_keys
            .iter()
            .filter(|k| k.get_wrapping_digest() == wrapped_key.get_wrapping_digest())
            .next()
            .is_some()
        {
            return Ok(false);
        }

        self.wrapped_keys.push(wrapped_key);
        Ok(true)
    }

    /// Remove the given key from this KeyStore, so it can no longer be used to
    /// open the KeyStore. Returns true if the key was removed, or false if the
    /// given key wasn't found in this KeyStore. It is an error to remove the
    /// last wrapping key from a KeyStore (doing so would leave it unopenable in
    /// the future).
    ///
    /// Note that it is possible to do this even if the KeyStore has no
    /// unwrapped master key (e.g., even if it has not been opened).
    pub fn remove_key<K: AbstractKey>(&mut self, key: &K) -> Result<bool> {
        if self.wrapped_keys.len() == 1 {
            if let Some(wrapped_key) = self.wrapped_keys.first() {
                if *wrapped_key.get_wrapping_digest() == key.get_digest() {
                    return Err(Error::Precondition(format!(
                        "refusing to remove all valid keys from this KeyStore"
                    )));
                }
            }
        }

        let original_length = self.wrapped_keys.len();
        let wrapped_keys = self
            .wrapped_keys
            .drain(..)
            .filter(|k| *k.get_wrapping_digest() != key.get_digest())
            .collect();
        self.wrapped_keys = wrapped_keys;
        Ok(original_length != self.wrapped_keys.len())
    }

    /// Return an immutable iterator over this KeyStore's wrapped keys. This
    /// may be useful to figure out which key to try to open with, for example,
    /// by checking the keys' signatures.
    ///
    /// This works even if the KeyStore has no unwrapped master key (e.g., even
    /// if it has not been opened).
    pub fn iter_wrapped_keys(&self) -> impl Iterator<Item = &WrappedKey> {
        self.wrapped_keys.iter()
    }
}

fn persist_key_store<P: AsRef<Path>>(path: P, keystore: &KeyStore) -> Result<()> {
    if !keystore.is_persistable() {
        return Err(Error::Precondition(format!(
            "KeyStore with no wrapping keys cannot be persisted"
        )));
    }

    let mut f = fs::File::create(path.as_ref())?;
    let data = keystore.to_vec()?;
    f.write_all(data.as_slice())?;
    Ok(())
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
    /// Construct a new DiskKeyStore instance. If the given path already exists,
    /// a pre-existing key store will be loaded from it. Otherwise, a brand new
    /// key store will be initialized.
    ///
    /// It is up to the caller to check for the latter case, and to add a key
    /// as appropriate.
    ///
    /// If `force_overwrite` is set to `true`, then a fresh instance is created,
    /// even if a previous one already existed.
    pub fn new<P: AsRef<Path>>(path: P, force_overwrite: bool) -> Result<Self> {
        let mut f = fs::OpenOptions::new()
            .read(true)
            // Open in write mode, in case we need to create the file.
            .write(true)
            // We'll always create it, if it doesn't already exist.
            .create(true)
            // Only truncate existing data if `force_overwrite` is set.
            .truncate(force_overwrite)
            .open(path.as_ref())?;

        Ok(DiskKeyStore {
            path: path.as_ref().to_path_buf(),
            inner: if f.metadata()?.len() == 0 {
                // If the file was of zero length, just remove it. Most likely
                // we created it, but if this key store doens't end up being
                // persisted we don't want to leave an orphaned file around.
                fs::remove_file(path.as_ref())?;
                KeyStore::new()?
            } else {
                KeyStore::load_read(&mut f)?
            },
        })
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
        if let Err(e) = persist_key_store(&self.path, &self.inner) {
            error!("{} (KeyStore {})", e, self.inner.get_id());
        }
    }
}
