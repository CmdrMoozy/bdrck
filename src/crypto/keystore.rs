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

#[derive(Deserialize, Serialize)]
struct KeyStoreContents {
    pub token_nonce: Option<Nonce>,
    pub token: Vec<u8>,
    pub wrapped_keys: Vec<WrappedKey>,
}

impl KeyStoreContents {
    pub fn new(master_key: &Key) -> Result<KeyStoreContents> {
        let (nonce, ciphertext) = master_key.encrypt(AUTH_TOKEN_CONTENTS.as_slice())?;
        Ok(KeyStoreContents {
            token_nonce: nonce,
            token: ciphertext,
            wrapped_keys: Vec::new(),
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<KeyStoreContents> {
        let file = File::open(path)?;
        Ok(msgpack::from_read(file)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = msgpack::to_vec(self)?;
        let mut file = File::create(path)?;
        Ok(file.write_all(data.as_slice())?)
    }

    pub fn is_master_key(&self, key: &Key) -> bool {
        let decrypted = match key.decrypt(self.token_nonce.as_ref(), self.token.as_slice()) {
            Err(_) => return false,
            Ok(d) => d,
        };
        decrypted.as_slice() == AUTH_TOKEN_CONTENTS.as_slice()
    }

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

pub struct KeyStore {
    path: PathBuf,
    master_key: Key,
    contents: KeyStoreContents,
}

impl KeyStore {
    fn new<P: AsRef<Path>>(path: P) -> Result<KeyStore> {
        let master_key = Key::new_random()?;
        let contents = KeyStoreContents::new(&master_key)?;

        Ok(KeyStore {
            path: path.as_ref().to_path_buf(),
            master_key: master_key,
            contents: contents,
        })
    }

    fn open<P: AsRef<Path>, K: AbstractKey>(path: P, key: &K) -> Result<KeyStore> {
        let contents = KeyStoreContents::open(path.as_ref())?;
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
            bail!("Failed to unwrap master key with the provided wrapping key");
        }

        Ok(KeyStore {
            path: path.as_ref().to_path_buf(),
            master_key: master_key.unwrap(),
            contents: contents,
        })
    }

    pub fn open_or_new<P: AsRef<Path>, K: AbstractKey>(path: P, key: &K) -> Result<KeyStore> {
        if path.as_ref().exists() {
            Self::open(path, key)
        } else {
            let mut keystore = Self::new(path)?;
            keystore.add_key(key)?;
            Ok(keystore)
        }
    }

    pub fn get_master_key(&self) -> &Key {
        &self.master_key
    }

    pub fn add_key<K: AbstractKey>(&mut self, key: &K) -> Result<bool> {
        Ok(self.contents.add_key(self.master_key.clone().wrap(key)?))
    }

    pub fn remove_key<K: AbstractKey>(&mut self, key: &K) -> Result<bool> {
        self.contents.remove_key(key)
    }
}

impl Drop for KeyStore {
    fn drop(&mut self) {
        self.contents.save(self.path.as_path()).unwrap();
    }
}
