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

use crate::error::*;
use crate::flags::*;
use std::net::IpAddr;
use std::path::PathBuf;

#[test]
fn test_command_callback() {
    #[command_callback]
    fn test_callback(a: PathBuf, b: String, c: Option<String>, d: &[IpAddr]) -> Result<()> {
        println!("{:?}", a);
        println!("{:?}", b);
        println!("{:?}", c);
        println!("{:?}", d);
        Ok(())
    }
}
