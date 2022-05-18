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

use crate::crypto::digest::*;
use crate::crypto::key::*;
use crate::crypto::secret::Secret;
use libc::{c_ulonglong, c_void};
use rmp_serde;

fn randombytes_into(buf: &mut [u8]) {
    unsafe {
        halite_sys::randombytes_buf(buf.as_mut_ptr() as *mut c_void, buf.len() as c_ulonglong);
    }
}

fn clone_key(key: &Key) -> Key {
    Key::deserialize(key.serialize().unwrap()).unwrap()
}

fn new_password(password: &str) -> Secret {
    let bytes = password.as_bytes();
    let mut s = Secret::with_len(bytes.len()).unwrap();
    unsafe { s.as_mut_slice() }.copy_from_slice(bytes);
    s
}

fn random_secret(len: usize) -> Secret {
    let mut s = Secret::with_len(len).unwrap();
    randombytes_into(unsafe { s.as_mut_slice() });
    s
}

#[test]
fn test_nonce_increment() {
    let nonce = Nonce::new();
    let next = Nonce::new();
    assert_eq!(nonce, next);
    assert_ne!(nonce, next.increment());
}

#[test]
fn test_digest_serde_round_trip() {
    let key = Key::new_random().unwrap();
    let digest = key.get_digest();
    let serialized = rmp_serde::to_vec(&digest).unwrap();
    let deserialized: Digest = rmp_serde::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(digest, deserialized);
}

#[test]
fn test_password_key_derivation() {
    let salt = Salt::default();
    let _key = Key::new_password(
        &new_password("foobar"),
        &salt,
        OPS_LIMIT_INTERACTIVE,
        MEM_LIMIT_INTERACTIVE,
    )
    .unwrap();
}

#[test]
fn test_basic_key_digest_comparison() {
    let a = Key::new_random().unwrap();
    let b = Key::new_random().unwrap();
    let c = clone_key(&a);

    assert_eq!(a.get_digest(), c.get_digest());
    assert_ne!(a.get_digest(), b.get_digest());
}

#[test]
fn test_encryption_roundtrip() {
    let key = Key::new_random().unwrap();
    let plaintext = random_secret(1024);
    let (nonce, ciphertext) = key.encrypt(&plaintext, None).unwrap();
    assert_ne!(unsafe { plaintext.as_slice() }, ciphertext.as_slice());
    let decrypted = key.decrypt(nonce.as_ref(), ciphertext.as_slice()).unwrap();
    assert_eq!(unsafe { plaintext.as_slice() }, unsafe {
        decrypted.as_slice()
    });
}

#[test]
fn test_decrypting_with_wrong_key_fails() {
    let key = Key::new_random().unwrap();
    let plaintext = random_secret(1024);
    let (nonce, ciphertext) = key.encrypt(&plaintext, None).unwrap();
    assert_ne!(unsafe { plaintext.as_slice() }, ciphertext.as_slice());

    let wrong_key = Key::new_random().unwrap();
    let decrypted_result = wrong_key.decrypt(nonce.as_ref(), ciphertext.as_slice());
    assert!(decrypted_result.is_err());
}
