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

use crate::error::Result;
use libc::{c_int, c_long, c_void};
use log::error;

// Not included in the libc crate yet, so hardcode it here.
#[allow(non_upper_case_globals)]
const SYS_memfd_secret: c_long = 447;

fn memfd_secret() -> Result<c_int> {
    let ret = unsafe { libc::syscall(SYS_memfd_secret, libc::O_CLOEXEC) };
    if ret < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(ret as c_int)
}

fn ftruncate(fd: c_int, len: usize) -> Result<()> {
    let ret = unsafe { libc::ftruncate64(fd, len as libc::off64_t) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

fn mmap(fd: c_int, len: usize) -> Result<*mut c_void> {
    let ret = unsafe {
        libc::mmap64(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if ret == libc::MAP_FAILED {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(ret)
}

fn munmap(ptr: *mut c_void, len: usize) -> Result<()> {
    let ret = unsafe { libc::munmap(ptr, len) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

fn close(fd: c_int) -> Result<()> {
    let ret = unsafe { libc::close(fd) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

/// Secret is somewhat like a Vec<u8>, but for sensitive data. It guarantees that its contents
/// won't be swapped out, and it also guarantees that the contents won't be visible to any other
/// process, or even the kernel.
///
/// NOTE: This requires a fairly recent kernel (5.14+), with CONFIG_SECRETMEM enabled. Currently
/// there is no fallback implementation, so if requirements aren't met, then constructing Secrets
/// will simply return an error.
///
/// NOTE: Memory allocated this way *does* count towards RLIMIT_MEMLOCK. In modern kernels this
/// defaults to 8 MiB, but it may perhaps need to be increased depending on how you're using this.
pub struct Secret {
    fd: c_int,
    ptr: *mut c_void,
    len: usize,
}

impl Drop for Secret {
    fn drop(&mut self) {
        if let Err(e) = self.clear() {
            error!(
                "Secret failed to clean up, memory and/or file descriptor leaked: {:?}",
                e
            );
        }
    }
}

impl Default for Secret {
    fn default() -> Self {
        Secret {
            fd: -1,
            ptr: std::ptr::null_mut(),
            len: 0,
        }
    }
}

impl Secret {
    fn clear(&mut self) -> Result<()> {
        if !self.ptr.is_null() {
            munmap(self.ptr, self.len)?;
            self.ptr = std::ptr::null_mut();
        }

        if self.fd != -1 {
            close(self.fd)?;
            self.fd = -1;
        }

        Ok(())
    }

    /// Create a new Secret buffer, initially with length zero. Before the buffer can be
    /// meaningfully used, resize will have to be called.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Secret buffer with the given initial length. The given initial length can be
    /// zero.
    pub fn with_len(len: usize) -> Result<Self> {
        let mut s = Secret::new();

        if len > 0 {
            s.fd = memfd_secret()?;
            ftruncate(s.fd, len)?;
            s.ptr = mmap(s.fd, len)?;
            s.len = len;
        }

        Ok(s)
    }

    /// Resize the buffer's length in bytes. If the new length is smaller, the existing data is
    /// truncated. If the new length is larger, the new bytes will be zeros.
    pub fn resize(&mut self, len: usize) -> Result<()> {
        /*
         * memfd_secret fds are *not* resizable! In fact, doing so might panic the kernel:
         * https://patchwork.kernel.org/project/linux-mm/patch/20220324210909.1843814-1-axelrasmussen@google.com/
         *
         * So, construct a new one, copy into it, and then replace ourself with it.
         */

        let mut next = Secret::with_len(len)?;

        {
            let copy_len = std::cmp::min(self.len, len);
            let (to_copy, to_zero) = unsafe { next.as_mut_slice() }.split_at_mut(copy_len);

            to_copy.copy_from_slice(unsafe { self.as_slice() }.split_at(copy_len).0);
            to_zero.fill(0);
        }

        self.clear()?;
        *self = std::mem::take(&mut next);
        Ok(())
    }

    /// Return this buffer's length in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a pointer to this Secret's underlying memory. The returned pointer is guaranteed to
    /// be suitable for constructing a slice, even if this Secret is empty. This pointer is
    /// guaranteed to be non-NULL.
    pub unsafe fn slice_ptr(&self) -> *mut u8 {
        let ret = if self.len > 0 {
            debug_assert!(!self.ptr.is_null());
            self.ptr as *mut u8
        } else {
            std::ptr::NonNull::dangling().as_ptr()
        };
        debug_assert!(!ret.is_null());
        ret
    }

    /// Access the underlying secret data. This function is unsafe primarily because you're
    /// touching secrets that shouldn't be exposed, so be very careful what you do with the data!
    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.slice_ptr(), self.len)
    }

    /// Mutably access the underlying secret data. This function is unsafe primarily because
    /// you're touching secrets that shouldn't be exposed, so be very careful what you do with
    /// the data!
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.slice_ptr(), self.len)
    }

    /// Try to copy this Secret's contents into a new Secret.
    pub fn try_clone(&self) -> Result<Self> {
        let mut other = Secret::with_len(self.len())?;
        unsafe { other.as_mut_slice().copy_from_slice(self.as_slice()) }
        Ok(other)
    }
}

unsafe impl Send for Secret {}
unsafe impl Sync for Secret {}
