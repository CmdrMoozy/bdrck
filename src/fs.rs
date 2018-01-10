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

use error::*;
use std::fs;
use std::path::Path;

/// This is a simple utility to create a new empty file. If a file at the given
/// path already exists, it will be truncated. It's an error if the path
/// already exists but is, for example, a directory.
pub fn create_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let _f = fs::File::create(path)?;
    Ok(())
}

/// An implementation of a function to create symbolic links on UNIX-style
/// OSes. This works equivalently to "ln -s target symlink".
#[cfg(not(target_os = "windows"))]
pub fn create_symlink<T: AsRef<Path>, S: AsRef<Path>>(target: T, symlink: S) -> Result<()> {
    Ok(::std::os::unix::fs::symlink(target, symlink)?)
}

/// An implementation of a fucntion to create symbolic links on Windows.
/// Windows has some weird policies here, in that this usually requires
/// administrator rights, and the underlying function call is different for
/// files and directories. This function attempts to handle all of this, but
/// the runtime semantics may differ from UNIX.
#[cfg(target_os = "windows")]
pub fn create_symlink<T: AsRef<Path>, S: AsRef<Path>>(target: T, symlink: S) -> Result<()> {
    Ok(if target.as_ref().is_dir() {
        ::std::os::windows::fs::symlink_dir(target, symlink)?
    } else {
        ::std::os::windows::fs::symlink_file(target, symlink)?
    })
}
