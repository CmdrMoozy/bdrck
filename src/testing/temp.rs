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
use fs::{create_file, create_symlink};
use std::fs;
use std::path::{Path, PathBuf};
use tempdir;

/// A directory within the system's standard temp directory that is
/// automatically deleted when it goes out of scope. The directory is created
/// on construction.
///
/// NOTE: For various reasons (e.g. races), temporary directories and files can be very dangerous
/// to rely upon in production code. This struct, as well as File which is based upon it, are
/// primarily intended to be used for unit testing only (thus their placement in the testing
/// submodule).
pub struct Dir {
    dir: tempdir::TempDir,
}

impl Dir {
    pub fn new(prefix: &str) -> Result<Dir> {
        Ok(Dir {
            dir: tempdir::TempDir::new(prefix)?,
        })
    }

    pub fn path(&self) -> &Path { self.dir.path() }

    /// A convenience function which adds the given relative path to this
    /// temporary directory's absolute path.
    pub fn sub_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        if path.as_ref().is_absolute() {
            bail!(
                "Cannot add absolute path '{}' to temporary directory path",
                path.as_ref().display()
            );
        }
        let mut buf = self.path().to_path_buf();
        buf.push(path);
        Ok(buf)
    }

    pub fn close(self) -> Result<()> { Ok(self.dir.close()?) }
}

/// A file within the system's standard temp directory that is automatically
/// deleted when it goes out of scope.
pub struct File {
    _dir: Option<Dir>,
    path: PathBuf,
}

impl File {
    /// Create a new temporary file within the standard system temporary
    /// directory.
    pub fn new_file() -> Result<File> {
        let dir = Dir::new("bdrck")?;
        let path = dir.sub_path("tempfile")?;
        let ret = File {
            _dir: Some(dir),
            path: path,
        };
        create_file(ret.path.as_path())?;
        Ok(ret)
    }

    /// Create a new temporary symlink within the standard system temporary
    /// directory, pointing at the given target.
    pub fn new_symlink<T: AsRef<Path>>(target: T) -> Result<File> {
        let dir = Dir::new("bdrck")?;
        let path = dir.sub_path("tempfile")?;
        let ret = File {
            _dir: Some(dir),
            path: path,
        };
        create_symlink(target, ret.path.as_path())?;
        Ok(ret)
    }

    /// Create a new temporary file at the specified path.
    pub fn new_file_at<P: AsRef<Path>>(path: P) -> Result<File> {
        let ret = File {
            _dir: None,
            path: path.as_ref().to_path_buf(),
        };
        create_file(ret.path.as_path())?;
        Ok(ret)
    }

    /// Create a new temporary symlink at the specified path, pointing at the
    /// given target.
    pub fn new_symlink_at<T: AsRef<Path>, S: AsRef<Path>>(target: T, symlink: S) -> Result<File> {
        let ret = File {
            _dir: None,
            path: symlink.as_ref().to_path_buf(),
        };
        create_symlink(target, symlink)?;
        Ok(ret)
    }

    pub fn path(&self) -> &Path { self.path.as_path() }

    fn close_impl(&self) -> Result<()> { Ok(fs::remove_file(self.path.as_path())?) }

    pub fn close(self) -> Result<()> { self.close_impl() }
}

impl Drop for File {
    #[allow(unused_must_use)]
    fn drop(&mut self) { self.close_impl(); }
}
