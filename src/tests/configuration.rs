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

use crate::configuration;
use crate::testing::temp;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TestConfiguration {
    foo: String,
}

lazy_static! {
    static ref TEST_IDENTIFIER: configuration::Identifier = configuration::Identifier {
        application: "bdrck_config".to_owned(),
        name: "test".to_owned(),
    };
}

#[test]
fn test_persistence() {
    crate::init().unwrap();

    let file = temp::File::new_file().unwrap();
    let path: path::PathBuf = file.path().to_owned();
    // Remove the file: an empty file isn't a valid serialized configuration struct.
    fs::remove_file(path.as_path()).unwrap();

    // Test that creating a configuration with an nonexistent file uses the default.
    let default = TestConfiguration {
        foo: "this is test data".to_owned(),
    };
    configuration::new(
        TEST_IDENTIFIER.clone(),
        default.clone(),
        Some(path.as_path()),
    )
    .ok()
    .unwrap();
    assert_eq!(default, configuration::get(&TEST_IDENTIFIER).ok().unwrap());

    // Test that when we update the configuration, the new version is persisted,
    // and is re-loaded upon recreation.
    let updated = TestConfiguration {
        foo: "this is some other test data".to_owned(),
    };
    configuration::set(&TEST_IDENTIFIER, updated.clone())
        .ok()
        .unwrap();
    assert_eq!(updated, configuration::get(&TEST_IDENTIFIER).ok().unwrap());
    configuration::remove::<TestConfiguration>(&TEST_IDENTIFIER)
        .ok()
        .unwrap();
    configuration::new(
        TEST_IDENTIFIER.clone(),
        default.clone(),
        Some(path.as_path()),
    )
    .ok()
    .unwrap();
    assert_eq!(updated, configuration::get(&TEST_IDENTIFIER).ok().unwrap());

    // Test that we can then reset back to defaults.
    configuration::reset::<TestConfiguration>(&TEST_IDENTIFIER)
        .ok()
        .unwrap();
    assert_eq!(default, configuration::get(&TEST_IDENTIFIER).ok().unwrap());
}
