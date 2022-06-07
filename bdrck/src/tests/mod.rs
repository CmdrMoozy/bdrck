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

#[cfg(test)]
mod cli;
#[cfg(test)]
mod configuration;
#[cfg(test)]
mod crypto;
#[cfg(test)]
mod fs;
#[cfg(test)]
mod http;
#[cfg(test)]
mod io;
#[cfg(test)]
mod logging;
#[cfg(test)]
mod net;
#[cfg(test)]
mod testing;

#[test]
fn test_all_tests_initialize_library() {
    use std::io::BufRead;

    crate::init().unwrap();

    // Our library has a global initialization function. The contract is, it must be called before
    // any other library functions are called, or undefined behavior may result. Since our tests
    // call our library functions, they too should initialize the library. :) But, it's very easy
    // to forget to add this initializer to new tests.
    //
    // This test reads our test source code, and checks that the number of instances of "#[test]"
    // matches the number of instances of "crate::init().unwrap();".

    let mut dir = std::env::current_exe()
        .unwrap()
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    while !dir.join("Cargo.toml").is_file() {
        assert!(dir.pop());
    }

    let src_dir = dir.join("bdrck/src");
    assert!(
        src_dir.is_dir(),
        "expected bdrck/src/ directory to exist: {}",
        src_dir.display()
    );

    let mut nr_files = 0;
    let mut dirs = vec![dir.join("bdrck/src/tests")];
    while !dirs.is_empty() {
        let dir = dirs.pop().unwrap();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let has_rs_extension = if let Some(extension) = path.extension() {
                extension == "rs"
            } else {
                false
            };

            if has_rs_extension && path.is_file() {
                nr_files += 1;

                let file = std::fs::File::open(&path).unwrap();
                let reader = std::io::BufReader::new(file);

                let mut nr_tests = 0;
                let mut nr_inits = 0;

                for line in reader.lines() {
                    let line = line.unwrap();
                    let trimmed = line.trim();

                    if trimmed == "#[test]" {
                        nr_tests += 1;
                    } else if trimmed == "crate::init().unwrap();" {
                        nr_inits += 1;
                    }
                }

                assert_eq!(
                    nr_tests,
                    nr_inits,
                    "expected file {} to have matching test ({}) and init() ({}) counts",
                    path.display(),
                    nr_tests,
                    nr_inits
                );
            } else if path.is_dir() {
                dirs.push(path.to_path_buf());
            }
        }
    }

    assert!(nr_files > 0, "expected to find > 0 test source files");
}
