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

use fs::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use testing::temp;

#[test]
fn test_path_bytes_round_trip() {
    let expected_path = PathBuf::from("/tmp/test_path");
    let bytes = path_to_bytes(expected_path.as_path()).unwrap();
    let path = path_from_bytes(bytes).unwrap();
    assert_eq!(expected_path, path);
}

#[test]
fn test_create_file() {
    let dir = temp::Dir::new("bdrck").unwrap();
    let mut file_path = dir.path().to_path_buf();
    file_path.push("test_file");
    assert_eq!(file_path.file_name().unwrap(), "test_file");
    assert!(!file_path.exists());
    create_file(file_path.as_path()).unwrap();
    assert!(file_path.exists());
    assert!(file_path.is_file());
}

#[test]
fn test_create_symlink() {
    const TEST_CONTENTS: &'static str = "this is a test";

    let dir = temp::Dir::new("bdrck").unwrap();

    let file_path = dir.path().join("test_file");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(TEST_CONTENTS.as_bytes()).unwrap();
    f.flush().unwrap();

    let symlink_path = dir.path().join("test_symlink");
    create_symlink(&file_path, &symlink_path).unwrap();
    assert!(
        fs::symlink_metadata(&symlink_path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let mut contents = String::new();
    let mut f = File::open(&symlink_path).unwrap();
    assert_eq!(
        TEST_CONTENTS.len(),
        f.read_to_string(&mut contents).unwrap()
    );
    assert_eq!(TEST_CONTENTS, contents.as_str());
}

#[test]
fn test_set_permissions_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp_file = temp::File::new_file().unwrap();
    set_permissions_mode(temp_file.path(), 0o444).unwrap();
    assert_eq!(
        0o444,
        fs::metadata(temp_file.path()).unwrap().permissions().mode() & 0x1FF
    );
    set_permissions_mode(temp_file.path(), 0o666).unwrap();
    assert_eq!(
        0o666,
        fs::metadata(temp_file.path()).unwrap().permissions().mode() & 0x1FF
    );
}
