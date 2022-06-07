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

use crate::io::*;
use crate::testing::temp;
use std::fs;

#[test]
fn test_read_at_most() {
    crate::init().unwrap();

    let testdata = b"Hello, world!";
    for buffer_size in &[testdata.len() - 1, testdata.len(), testdata.len() + 1] {
        let tf = temp::File::new_file().unwrap();
        fs::write(tf.path(), testdata).unwrap();

        let mut f = fs::File::open(tf.path()).unwrap();
        let result = read_at_most(&mut f, *buffer_size);

        if *buffer_size < testdata.len() {
            assert!(result.is_err());
        } else {
            let data = result.unwrap();
            assert_eq!(testdata, data.as_slice());
        }
    }
}
