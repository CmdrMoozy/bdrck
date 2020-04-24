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
use std::io::{self, Read};

/// Reads from the givne `Read` until the buffer is filled. If EOF is reached
/// first, this is fine. If we hit EOF exactly when the buffer is filled, that's
/// also fine.
///
/// However, if there are bytes remaining after the buffer is filled (we didn't
/// hit EOF), this is considered an error.
///
/// This function is useful when you are reading e.g. user-provided input of
/// unknown size, and you want to place an upper bound on it (e.g. to avoid
/// OOMs.)
///
/// NOTE: A limitation here is that, if buf is *exactly* big enough to hold the
/// data, we must read one extra byte past there (which is then discarded). This
/// means you can't rely on continuing to use the `Read` after calling this
/// function. For the intended use cases, this is not a problem, but it needs to
/// be kept in mind.
pub fn read_at_most_into<R: Read>(r: &mut R, mut buf: &mut [u8]) -> Result<usize> {
    let maximum_bytes: usize = buf.len();
    let mut read_bytes: usize = 0;
    let mut hit_eof: bool = false;

    while !buf.is_empty() {
        match r.read(buf) {
            // We know `buf` was nonempty, so a return value of 0 means EOF.
            Ok(0) => {
                hit_eof = true;
                break;
            }
            // We read some bytes. Update our buffer window, and continue.
            Ok(n) => {
                read_bytes += n;
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            // Interrupted is transient, and can be retried.
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            // If we hit any other error, halt and return it.
            Err(e) => return Err(e.into()),
        }
    }

    // If we didn't hit EOF naturally, check if we ended exactly at the end of
    // the file.
    if !hit_eof {
        match r.bytes().next() {
            // We hit EOF (there are no more bytes).
            None => {
                hit_eof = true;
            }
            // There is more data. We didn't hit EOF. This becomes an error
            // below.
            Some(Ok(_)) => {}
            // If we hit a non-EOF error, propagate it. We can't guarantee
            // whether we hit EOF or not if this happens.
            Some(Err(e)) => return Err(e.into()),
        };
    }

    if !hit_eof {
        return Err(Error::InputTooBig(format!(
            "refusing to read more bytes; expected at most {}",
            maximum_bytes
        )));
    }

    Ok(read_bytes)
}

/// A convenience wrapper around `read_at_most_into`, which allocates its own
/// buffer.
///
/// This is most useful when you, for example, would normally want
/// `read_to_end`, but with an upper bound on how many bytes you're willing to
/// read.
pub fn read_at_most<R: Read>(r: &mut R, maximum_bytes: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0; maximum_bytes];
    let bytes_read = read_at_most_into(r, buf.as_mut_slice())?;
    buf.truncate(bytes_read);
    Ok(buf)
}
