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

use crate::testing::temp::*;
use std::fs;
use std::io::{Read, Write};

#[test]
fn test_read_write_temp_file() {
    crate::init().unwrap();

    let temp_file = File::new_file().unwrap();
    let test_contents: String = "this is some arbitrary test data".to_owned();

    // Write some data to the temporary file.
    {
        let mut file = fs::File::create(temp_file.path()).unwrap();
        file.write_all(test_contents.as_bytes()).unwrap();
    }

    // Read the data back and make sure that works.
    let mut file = fs::File::open(temp_file.path()).unwrap();
    let mut read_contents = String::new();
    file.read_to_string(&mut read_contents).unwrap();
    assert_eq!(test_contents, read_contents);
}

#[test]
fn test_new_file_in_subdirectory() {
    crate::init().unwrap();

    let dir = Dir::new("bdrck").unwrap();
    let file = File::new_file_at(dir.sub_path("foo/bar/file.txt").unwrap()).unwrap();
    assert!(file.path().exists());
}

#[test]
fn test_new_symlink_in_subdirectory() {
    crate::init().unwrap();

    let dir = Dir::new("bdrck").unwrap();
    let file = File::new_file_at(dir.sub_path("foo/bar/file.txt").unwrap()).unwrap();
    let symlink =
        File::new_symlink_at(file.path(), dir.sub_path("bar/baz/symlink.txt").unwrap()).unwrap();
    assert!(symlink.path().exists());
}
