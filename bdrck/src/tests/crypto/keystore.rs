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

use crate::crypto::key::*;
use crate::crypto::keystore::*;
use crate::testing::temp;
use std::fs;

#[test]
fn test_keystore_save_round_trip() {
    let file = temp::File::new_file().unwrap();

    let wrap_key = Key::new_random().unwrap();
    let master_key: Option<Key>;

    {
        let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
        keystore.add_key(&wrap_key).unwrap();
        master_key = Some(keystore.get_master_key().unwrap().clone());
    }

    {
        let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
        keystore.open(&wrap_key).unwrap();
        assert_eq!(
            master_key.as_ref().unwrap().get_digest(),
            keystore.get_master_key().unwrap().get_digest()
        );
    }
}

#[test]
fn test_keystore_open_with_added_key() {
    let file = temp::File::new_file().unwrap();

    let salt = Salt::default();
    let keya = Key::new_password(
        "foo".as_bytes(),
        &salt,
        OPS_LIMIT_INTERACTIVE,
        MEM_LIMIT_INTERACTIVE,
    )
    .unwrap();
    let keyb = Key::new_password(
        "bar".as_bytes(),
        &salt,
        OPS_LIMIT_INTERACTIVE,
        MEM_LIMIT_INTERACTIVE,
    )
    .unwrap();
    assert_ne!(keya.get_digest(), keyb.get_digest());
    let master_key: Option<Key>;

    {
        let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
        keystore.add_key(&keya).unwrap();
        master_key = Some(keystore.get_master_key().unwrap().clone());
        assert!(keystore.add_key(&keyb).unwrap());
    }

    {
        let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
        keystore.open(&keyb).unwrap();
        assert_eq!(
            master_key.as_ref().unwrap().get_digest(),
            keystore.get_master_key().unwrap().get_digest()
        );
    }
}

#[test]
fn test_add_duplicate_key() {
    let file = temp::File::new_file().unwrap();

    let wrap_key = Key::new_random().unwrap();
    // Note that creating a new DiskKeyStore automatically adds the given key.
    let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
    keystore.add_key(&wrap_key).unwrap();
    // Check that adding the same key again is a no-op (returns false).
    assert!(!keystore.add_key(&wrap_key).unwrap());
}

#[test]
fn test_remove_unused_key() {
    let file = temp::File::new_file().unwrap();

    let salt = Salt::default();
    let keya = Key::new_password(
        "foo".as_bytes(),
        &salt,
        OPS_LIMIT_INTERACTIVE,
        MEM_LIMIT_INTERACTIVE,
    )
    .unwrap();
    let keyb = Key::new_password(
        "bar".as_bytes(),
        &salt,
        OPS_LIMIT_INTERACTIVE,
        MEM_LIMIT_INTERACTIVE,
    )
    .unwrap();
    assert_ne!(keya.get_digest(), keyb.get_digest());

    let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
    keystore.add_key(&keya).unwrap();
    // Test that removing some other key returns false, since it isn't in the
    // DiskKeyStore.
    assert!(!keystore.remove_key(&keyb).unwrap());
}

#[test]
fn test_remove_only_key() {
    let file = temp::File::new_file().unwrap();

    let key = Key::new_random().unwrap();
    let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
    keystore.add_key(&key).unwrap();
    // Test that removing the sole key is treated as an error.
    assert!(keystore.remove_key(&key).is_err());
}

#[test]
fn test_remove_first_key() {
    let file = temp::File::new_file().unwrap();

    let key_a = Key::new_random().unwrap();
    let key_b = Key::new_random().unwrap();
    // Create a keystore with one initial key.
    let mut keystore = DiskKeyStore::new(file.path(), false).unwrap();
    keystore.add_key(&key_a).unwrap();
    // Add a second key.
    assert!(keystore.add_key(&key_b).unwrap());
    // Try removing the original key - this should succeed, since there is a
    // second valid key.
    assert!(keystore.remove_key(&key_a).unwrap());
}

#[test]
fn test_unpersistable() {
    let file = temp::File::new_file().unwrap();
    // Explicitly make sure the file doesn't exist.
    fs::remove_file(file.path()).unwrap();
    assert!(!file.path().exists());

    {
        // Construct a new key store, but don't add any keys.
        let keystore = DiskKeyStore::new(file.path(), false).unwrap();
        assert!(!keystore.is_persistable());
    }

    // Since the key store was not persistable, the file should still not exist.
    assert!(!file.path().exists());
}
