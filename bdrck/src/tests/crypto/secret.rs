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

use crate::crypto::secret::*;

#[test]
fn test_empty() {
    crate::init().unwrap();

    let mut sa = Secret::new();
    assert_eq!(0, sa.len());
    assert!(unsafe { sa.as_slice().is_empty() });
    assert!(unsafe { sa.as_mut_slice().is_empty() });

    let mut sb = Secret::with_len(0).unwrap();
    assert_eq!(0, sb.len());
    assert!(unsafe { sb.as_slice().is_empty() });
    assert!(unsafe { sb.as_mut_slice().is_empty() });
}

#[test]
fn test_roundtrip() {
    crate::init().unwrap();

    let data = "foo bar baz this is a test";
    let bytes = data.as_bytes();

    let mut s = Secret::with_len(bytes.len()).unwrap();
    assert_eq!(bytes.len(), s.len());
    unsafe {
        s.as_mut_slice().copy_from_slice(bytes);
    }

    let result = unsafe { std::str::from_utf8(s.as_slice()).unwrap() };
    assert_eq!(result, data);
}

#[test]
fn test_resize() {
    crate::init().unwrap();

    let mut data = "foobar foobar foobar".as_bytes().to_vec();
    let orig_len = data.len();

    let mut s = Secret::with_len(data.len()).unwrap();
    unsafe {
        s.as_mut_slice().copy_from_slice(data.as_slice());
    }

    data.resize(orig_len * 2, 0);
    s.resize(orig_len * 2).unwrap();

    assert_eq!(orig_len * 2, data.len());
    assert_eq!(data.len(), s.len());
    assert_eq!(data.as_slice(), unsafe { s.as_slice() });

    data.truncate(orig_len / 2);
    s.resize(orig_len / 2).unwrap();

    assert_eq!(orig_len / 2, data.len());
    assert_eq!(data.len(), s.len());
    assert_eq!(data.as_slice(), unsafe { s.as_slice() });

    s.resize(0).unwrap();

    assert_eq!(0, s.len());
    assert!(unsafe { s.as_slice().is_empty() });
    assert!(unsafe { s.as_mut_slice().is_empty() });

    s.resize(orig_len / 2).unwrap();
    unsafe {
        s.as_mut_slice().copy_from_slice(data.as_slice());
    }

    assert_eq!(orig_len / 2, data.len());
    assert_eq!(data.len(), s.len());
    assert_eq!(data.as_slice(), unsafe { s.as_slice() });
}
