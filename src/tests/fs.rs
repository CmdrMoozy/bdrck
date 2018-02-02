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
use testing::temp;

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
