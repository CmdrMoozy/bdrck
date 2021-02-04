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
use errno;
use libc;
use log::{debug, warn};
use std::ffi::{CString, OsString};
use std::fs::{self, Permissions};
use std::mem;
use std::path::{Path, PathBuf};
use std::ptr;

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
    use libc::c_int;

    let path_cstr = CString::new(path_to_bytes(path.as_ref())?)?;
    let ret: c_int = unsafe {
        match follow {
            false => libc::lchown(path_cstr.as_ptr(), uid, gid),
            true => libc::chown(path_cstr.as_ptr(), uid, gid),
        }
    };

    if ret == -1 {
        let error = errno();
        if (error.0 == libc::EACCES || error.0 == libc::EPERM) && !fail_on_access_denied {
            warn!(
                "Failed to change ownership of '{}': {}",
                path.as_ref().display(),
                error
            );
        } else {
            return Err(std::io::Error::from_raw_os_error(error.into()).into());
        }
    }

    Ok(())
}

/// This function is a safe implementation of chown(), effectively. On
/// Windows there is no concept of file ownership, so this function is a
/// no-op.
#[cfg(target_os = "windows")]
pub fn set_ownership<P: AsRef<Path>>(_: P, _: u32, _: u32, _: bool, _: bool) -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[derive(Debug)]
enum SysconfBufferKind {
    User,
    Group,
}

#[cfg(not(target_os = "windows"))]
fn get_sysconf_buffer_size(kind: SysconfBufferKind) -> usize {
    let k = match kind {
        SysconfBufferKind::User => libc::_SC_GETPW_R_SIZE_MAX,
        SysconfBufferKind::Group => libc::_SC_GETGR_R_SIZE_MAX,
    };

    let mut s = unsafe { libc::sysconf(k) } as usize;

    /*
     * This platform might not have any real maximum, in which case -1 is returned. Check for this,
     * so we don't try to allocate a buffer of 2^64 bytes. :)
     *
     * In this case, we just default to 1024 bytes, which seems to be a common value returned by
     * most Linux libc implementations.
     *
     * Also, just for safety's sake, deal with any "unreasonably large" value.
     */
    if s == usize::MAX {
        debug!(
            "libc has no maximum {:?} buffer size; defaulting to 1024 bytes",
            kind
        );
        s = 1024;
    } else if s > 1024 * 1024 * 10 {
        debug!(
            "libc returned unreasonable {:?} buffer size ({} bytes); defaulting to 1024 bytes",
            kind, s
        );
        s = 1024;
    }

    s
}

/// Returns the UNIX uid for the user with the given name.
#[cfg(not(target_os = "windows"))]
fn lookup_uid(name: &str) -> Result<u32> {
    let mut passwd = unsafe {
        mem::transmute::<[u8; mem::size_of::<libc::passwd>()], libc::passwd>(
            [0_u8; mem::size_of::<libc::passwd>()],
        )
    };
    let mut passwd_ptr: *mut libc::passwd = ptr::null_mut();
    let cname = CString::new(name)?;
    let buf_len = get_sysconf_buffer_size(SysconfBufferKind::User);
    let mut buf = vec![0_i8; buf_len];
    let ret = unsafe {
        libc::getpwnam_r(
            cname.as_ptr(),
            &mut passwd,
            buf.as_mut_ptr(),
            buf_len,
            &mut passwd_ptr,
        )
    };
    if passwd_ptr.is_null() {
        if ret == 0
            || ret == libc::ENOENT
            || ret == libc::ESRCH
            || ret == libc::EBADF
            || ret == libc::EPERM
        {
            return Err(Error::NotFound(format!(
                "unrecognized user name '{}'",
                name
            )));
        } else {
            return Err(std::io::Error::from_raw_os_error(ret).into());
        }
    }
    Ok(passwd.pw_uid)
}

/// Returns the UNIX gid for the group with the given name.
#[cfg(not(target_os = "windows"))]
fn lookup_gid(name: &str) -> Result<u32> {
    let mut group = libc::group {
        gr_name: ptr::null_mut(),
        gr_passwd: ptr::null_mut(),
        gr_gid: 0,
        gr_mem: ptr::null_mut(),
    };
    let mut group_ptr: *mut libc::group = ptr::null_mut();
    let cname = CString::new(name)?;
    let buf_len = get_sysconf_buffer_size(SysconfBufferKind::Group);
    let mut buf = vec![0_i8; buf_len];
    let ret = unsafe {
        libc::getgrnam_r(
            cname.as_ptr(),
            &mut group,
            buf.as_mut_ptr(),
            buf_len,
            &mut group_ptr,
        )
    };
    if group_ptr.is_null() {
        if ret == 0
            || ret == libc::ENOENT
            || ret == libc::ESRCH
            || ret == libc::EBADF
            || ret == libc::EPERM
        {
            return Err(Error::NotFound(format!(
                "unrecognized group name '{}'",
                name
            )));
        } else {
            return Err(std::io::Error::from_raw_os_error(ret).into());
        }
    }
    Ok(group.gr_gid)
}

/// Set the user and group ownership of a file or directory. This is a
/// convenience wrapper around `set_ownership` which allows the user and group
/// to be specified by name instead of by ID.
#[cfg(not(target_os = "windows"))]
pub fn set_ownership_by_name<P: AsRef<Path>>(
    path: P,
    user: &str,
    group: &str,
    fail_on_access_denied: bool,
    follow: bool,
) -> Result<()> {
    set_ownership(
        path,
        lookup_uid(user)?,
        lookup_gid(group)?,
        fail_on_access_denied,
        follow,
    )
}

/// Set the user and group ownership of a file or directory. This is a
/// convenience wrapper around `set_ownership` which allows the user and group
/// to be specified by name instead of by ID.
#[cfg(target_os = "windows")]
pub fn set_ownership_by_name<P: AsRef<Path>>(
    _: P,
    _: &str,
    _: &str,
    _: bool,
    _: bool,
) -> Result<()> {
    Ok(())
}
