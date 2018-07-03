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

use crypto::key::*;
use msgpack;
use sodiumoxide::randombytes::randombytes;

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
    let serialized = msgpack::to_vec(&digest).unwrap();
    let deserialized: Digest = msgpack::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(digest, deserialized);
}

#[test]
fn test_password_key_derivation() {
    let salt = Salt::default();
    let _key = Key::new_password(
        "foobar".as_bytes(),
        &salt,
        OPS_LIMIT_INTERACTIVE,
        MEM_LIMIT_INTERACTIVE,
    ).unwrap();
}

#[test]
fn test_basic_key_digest_comparison() {
    let a = Key::new_random().unwrap();
    let b = Key::new_random().unwrap();
    let c = a.clone();

    assert_eq!(a.get_digest(), c.get_digest());
    assert_ne!(a.get_digest(), b.get_digest());
}

#[test]
fn test_encryption_roundtrip() {
    let key = Key::new_random().unwrap();
    let plaintext = randombytes(1024);
    let (nonce, ciphertext) = key.encrypt(plaintext.as_slice(), None).unwrap();
    assert_ne!(plaintext.as_slice(), ciphertext.as_slice());
    let decrypted = key.decrypt(nonce.as_ref(), ciphertext.as_slice()).unwrap();
    assert_eq!(plaintext, decrypted);
}

#[test]
fn test_decrypting_with_wrong_key_fails() {
    let key = Key::new_random().unwrap();
    let plaintext = randombytes(1024);
    let (nonce, ciphertext) = key.encrypt(plaintext.as_slice(), None).unwrap();
    assert_ne!(plaintext.as_slice(), ciphertext.as_slice());

    let wrong_key = Key::new_random().unwrap();
    let decrypted_result = wrong_key.decrypt(nonce.as_ref(), ciphertext.as_slice());
    assert!(decrypted_result.is_err());
}

#[test]
fn test_wrapping_roundtrip() {
    let a = Key::new_random().unwrap();
    let b = Key::new_random().unwrap();
    let c = Key::new_random().unwrap();

    let wrapped_once = a.clone().wrap(&b).unwrap();
    let wrapped_twice = wrapped_once.clone().wrap(&c).unwrap();
    let unwrapped_once = match wrapped_twice.unwrap(&c).unwrap() {
        WrappedPayload::Key(_) => panic!("Expected nested wrapped key, got raw key"),
        WrappedPayload::WrappedKey(w) => w,
    };
    let unwrapped = match unwrapped_once.unwrap(&b).unwrap() {
        WrappedPayload::Key(k) => k,
        WrappedPayload::WrappedKey(_) => panic!("Expected raw key, got nested wrapped key"),
    };
    assert_eq!(a.get_digest(), unwrapped.get_digest());
}

#[test]
fn test_unwrapping_with_wrong_key_fails() {
    let a = Key::new_random().unwrap();
    let b = Key::new_random().unwrap();
    let wrong_key = Key::new_random().unwrap();

    let wrapped = a.clone().wrap(&b).unwrap();
    assert!(wrapped.clone().unwrap(&wrong_key).is_err());

    let unwrapped = match wrapped.unwrap(&b).unwrap() {
        WrappedPayload::Key(k) => k,
        WrappedPayload::WrappedKey(_) => panic!("Expected raw key, got nested wrapped key"),
    };
    assert_eq!(a.get_digest(), unwrapped.get_digest());
}
