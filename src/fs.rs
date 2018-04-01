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
use std::ffi::OsString;
use std::fs::{self, Permissions};
use std::path::{Path, PathBuf};

/// Returns the given Path as a byte vector. This function may be useful for
/// some kinds of serialization, or for calling C functions.
#[cfg(not(target_os = "windows"))]
pub fn path_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;

    Ok(Vec::from(path.as_ref().as_os_str().as_bytes()))
}

/// Returns the given path as a byte vector. Note that on Windows OS strings
/// are UTF-16, so the returned bytes are *not* necessarily valid UTF-8.
/// However, is is guaranteed that the path can be reconstructed losslessly
/// via path_from_bytes.
#[cfg(target_os = "windows")]
pub fn path_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    use byteorder::{BigEndian, WriteBytesExt};
    use std::os::windows::ffi::OsStrExt;

    let chars: Vec<u16> = path.as_ref().as_os_str().encode_wide().collect();
    let mut bytes: Vec<u8> = Vec::with_capacity(chars.len() * 2);
    for c in chars {
        bytes.write_u16::<BigEndian>(c)?;
    }
    Ok(bytes)
}

/// Construct a PathBuf from its byte representation, for example as returned by
/// `path_to_bytes`.
#[cfg(not(target_os = "windows"))]
pub fn path_from_bytes(bytes: Vec<u8>) -> Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;

    Ok(PathBuf::from(OsString::from_vec(bytes)))
}

/// Construct a PathBuf from its UTF-16 byte representation. This function may
/// fail if converting to a vector of u16 fails.
#[cfg(target_os = "windows")]
pub fn path_from_bytes(mut bytes: Vec<u8>) -> Result<PathBuf> {
    use byteorder::{BigEndian, ReadBytesExt};
    use std::os::windows::ffi::OsStringExt;

    let mut chars: Vec<u16> = vec![0; bytes.len() / 2];
    bytes.read_u16_into::<BigEndian>(chars.as_mut_slice())?;
    Ok(PathBuf::from(OsString::from_wide(chars.as_slice())))
}

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

/// Set the permissions mode for the given file or directory. This is roughly
/// equivalent to `chmod(2)` on Linux. Note that UNIX-style systems do not
/// support changing mode for a symlink itself, so this function always follows
/// symlinks (see `man 2 fchmodat` for details).
#[cfg(not(target_os = "windows"))]
pub fn set_permissions_mode<P: AsRef<Path>>(path: P, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = Permissions::from_mode(mode);
    Ok(::std::fs::set_permissions(path, permissions)?)
}

/// This function sets the UNIX-style permissions mode of the file at the
/// given path. On Windows, this concept is generally not supported, so
/// this function is just a no-op. This function exists so callers can
/// deal with this in a platform-agnostic way.
#[cfg(target_os = "windows")]
pub fn set_permissions_mode<P: AsRef<Path>>(_: P, _: u32) -> Result<()> {
    Ok(())
}

/// This function is a safe wrapper around chown(). If fail_on_access_denied
/// is set to true, then an EACCES error is considered a failure, and we'll
/// return Err(...). Otherwise, this is considered a soft failure, and a warning
/// will be logged, but Ok(()) will still be returned.
#[cfg(not(target_os = "windows"))]
pub fn set_ownership<P: AsRef<Path>>(
    path: P,
    uid: u32,
    gid: u32,
    fail_on_access_denied: bool,
    follow: bool,
) -> Result<()> {
    use errno::errno;
    use libc::{self, c_int, gid_t, uid_t};
    use std::ffi::CString;

    let path_cstr = CString::new(::fs::path_to_bytes(path.as_ref())?)?;
    let ret: c_int = unsafe {
        match follow {
            false => libc::lchown(path_cstr.as_ptr(), uid as uid_t, gid as gid_t),
            true => libc::chown(path_cstr.as_ptr(), uid as uid_t, gid as gid_t),
        }
    };

    if ret == -1 {
        let error = errno();
        if error.0 == libc::EACCES && !fail_on_access_denied {
            warn!(
                "Failed to change ownership of '{}': access denied",
                path.as_ref().display()
            );
        } else {
            bail!(error.to_string());
        }
    }

    Ok(())
}

/// This function is a safe implementation of chown(), effectively. On
/// Windows there is no concept of file ownership, so this function is a
/// no-op.
#[cfg(target_os = "windows")]
pub fn set_ownership<P: AsRef<Path>>(_: P, _: u32, _: u32, _: bool) -> Result<()> {
    Ok(())
}
