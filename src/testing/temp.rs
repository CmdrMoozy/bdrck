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
use crate::fs::{create_file, create_symlink};
use failure::format_err;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const TEMP_DIR_NAME_RAND_CHARS: usize = 32;
const TEMP_DIR_RAND_RETRIES: usize = 1024;

/// A directory within the system's standard temp directory that is
/// automatically deleted when it goes out of scope. The directory is created
/// on construction.
///
/// NOTE: For various reasons (e.g. races), temporary directories and files can be very dangerous
/// to rely upon in production code. This struct, as well as File which is based upon it, are
/// primarily intended to be used for unit testing only (thus their placement in the testing
/// submodule).
pub struct Dir {
    path: PathBuf,
}

impl Dir {
    /// This is a shortcut version of new_in, which just creates the directory
    /// within the system's default temporary directory.
    pub fn new(prefix: &str) -> Result<Dir> {
        Dir::new_in(&env::temp_dir(), prefix)
    }

    /// Create a new temporary directory, as a subdirectory of the given other
    /// temporary directory, with the given prefix in its name. The prefix
    /// should generally be something application-specific, so if the temporary
    /// directory is somehow left over its origin can be identified.
    fn new_in<P: AsRef<Path>>(temp_dir: P, prefix: &str) -> Result<Dir> {
        let mut rng = thread_rng();
        for _ in 0..TEMP_DIR_RAND_RETRIES {
            let suffix: String = rng
                .sample_iter(&Alphanumeric)
                .take(TEMP_DIR_NAME_RAND_CHARS)
                .collect();
            let name = if prefix.is_empty() {
                suffix
            } else {
                format!("{}-{}", prefix, suffix)
            };
            let path = temp_dir.as_ref().join(&name);
            match fs::create_dir(&path) {
                Ok(_) => return Ok(Dir { path: path }),
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        return Err(Error::Io(::std::io::Error::new(
            ::std::io::ErrorKind::AlreadyExists,
            "Failed to find unique random temporary directory name",
        )));
    }

    /// Return the path to this temporary directory.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// A convenience function which adds the given relative path to this
    /// temporary directory's absolute path.
    pub fn sub_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        if path.as_ref().is_absolute() {
            return Err(Error::InvalidArgument(format_err!(
                "Cannot add absolute path '{}' to temporary directory path",
                path.as_ref().display()
            )));
        }
        Ok(self.path.as_path().join(path))
    }

    fn close_impl(&self) -> Result<()> {
        Ok(fs::remove_dir_all(&self.path)?)
    }

    /// "Close" this temporary directory, by deleting it along with all of its
    /// contents. This is called automatically by the Drop implementation, but
    /// it can also be called manually if you want to dispose of this instance
    /// without just letting it go out of scope.
    pub fn close(self) -> Result<()> {
        self.close_impl()
    }
}

impl Drop for Dir {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.close_impl();
    }
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
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
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
        if let Some(parent) = symlink.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        create_symlink(target, symlink)?;
        Ok(ret)
    }

    /// Return the path to this temporary file.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn close_impl(&self) -> Result<()> {
        Ok(fs::remove_file(self.path.as_path())?)
    }

    /// "Close" this temporary file by deleting it. This is called automatically
    /// by the Drop implementation, but it can also be called manually if you
    /// want to dispose of this instance without just letting it go out of
    /// scope.
    pub fn close(self) -> Result<()> {
        self.close_impl()
    }
}

impl Drop for File {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.close_impl();
    }
}
