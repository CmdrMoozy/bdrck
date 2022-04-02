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

use crate::crypto::key::{AbstractKey, Key};
use crate::crypto::wrap::*;

#[test]
fn test_wrapping_roundtrip() {
    let a = Key::new_random().unwrap();
    let b = Key::new_random().unwrap();

    let wrapped = WrappedKey::wrap(&a, &b).unwrap();
    assert_ne!(wrapped.get_digest(), a.get_digest());
    assert_eq!(wrapped.get_wrapping_digest(), &b.get_digest());

    let unwrapped: Key = wrapped.unwrap(&b).unwrap();
    assert_eq!(a.get_digest(), unwrapped.get_digest());
}

#[test]
fn test_unwrapping_with_wrong_key_fails() {
    let a = Key::new_random().unwrap();
    let b = Key::new_random().unwrap();
    let wrong_key = Key::new_random().unwrap();

    let wrapped = WrappedKey::wrap(&a, &b).unwrap();
    assert!(wrapped.unwrap::<Key, Key>(&wrong_key).is_err());
}
