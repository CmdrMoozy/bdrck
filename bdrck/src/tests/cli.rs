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

use crate::cli::*;
use std::collections::HashSet;
use std::io::{Read, Write};

// The write buffer size we preallocate, per instance of `TestStreamBuffers`.
const TEST_WRITE_BUFFER_SIZE_BYTES: usize = 1024 * 100;

#[derive(Clone)]
struct TestTerminalAttributes {
    on: HashSet<TerminalFlag>,
    off: HashSet<TerminalFlag>,
}

impl TestTerminalAttributes {
    fn new() -> Self {
        TestTerminalAttributes {
            on: [TerminalFlag::Echo].iter().cloned().collect(),
            off: HashSet::new(),
        }
    }
}

impl AbstractTerminalAttributes for TestTerminalAttributes {
    fn enable(&mut self, flag: TerminalFlag) {
        self.on.insert(flag);
        self.off.remove(&flag);
    }

    fn disable(&mut self, flag: TerminalFlag) {
        self.on.remove(&flag);
        self.off.insert(flag);
    }
}

struct BufferState {
    read_ptr: Option<(*const u8, *const u8)>,
    write_ptr: Option<(*mut u8, *mut u8)>,
}

struct TestStreamReader {
    state: *mut BufferState,
}

impl Read for TestStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        unsafe {
            let (current, end) = (*self.state).read_ptr.clone().unwrap();
            let remaining = end as usize - current as usize;
            let to_read = std::cmp::min(remaining, buf.len());
            std::ptr::copy_nonoverlapping(current, buf.as_mut_ptr(), to_read);
            (*self.state).read_ptr = Some((current.offset(to_read as isize), end));
            Ok(to_read)
        }
    }
}

struct TestStreamWriter {
    state: *mut BufferState,
}

impl Write for TestStreamWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        unsafe {
            let (current, end) = (*self.state).write_ptr.clone().unwrap();
            let remaining = end as usize - current as usize;
            let to_write = std::cmp::min(remaining, buf.len());
            if to_write < buf.len() {
                panic!(
                    "Attempted to write {} bytes, only {} bytes left in buffer",
                    buf.len(),
                    remaining
                );
            }
            std::ptr::copy_nonoverlapping(buf.as_ptr(), current, to_write);
            (*self.state).write_ptr = Some((current.offset(to_write as isize), end));
            Ok(to_write)
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

struct TestStream {
    isatty: bool,
    attributes: TestTerminalAttributes,
    state: *mut BufferState,
}

impl AbstractStream for TestStream {
    type Attributes = TestTerminalAttributes;

    fn isatty(&self) -> bool {
        self.isatty
    }

    fn get_attributes(&self) -> IoResult<Self::Attributes> {
        Ok(self.attributes.clone())
    }

    fn set_attributes(&mut self, attributes: &Self::Attributes) -> IoResult<()> {
        self.attributes = attributes.clone();
        Ok(())
    }

    fn as_reader(&self) -> Option<Box<dyn Read>> {
        unsafe {
            if (*self.state).read_ptr.is_some() {
                Some(Box::new(TestStreamReader { state: self.state }))
            } else {
                None
            }
        }
    }

    fn as_writer(&self) -> Option<Box<dyn Write>> {
        unsafe {
            if (*self.state).write_ptr.is_some() {
                Some(Box::new(TestStreamWriter { state: self.state }))
            } else {
                None
            }
        }
    }
}

struct TestStreamBuffers {
    read_buffer: Option<Vec<u8>>,
    write_buffer: Option<Vec<u8>>,
    state: BufferState,
}

impl TestStreamBuffers {
    fn new(read_input: Option<String>, support_write: bool) -> Self {
        let read_buffer = read_input.map(|s| s.into_bytes());
        let mut write_buffer = match support_write {
            false => None,
            true => Some(Vec::with_capacity(TEST_WRITE_BUFFER_SIZE_BYTES)),
        };

        let state = BufferState {
            read_ptr: read_buffer
                .as_ref()
                .map(|b| (b.as_ptr(), unsafe { b.as_ptr().offset(b.len() as isize) })),
            write_ptr: write_buffer.as_mut().map(|b: &mut Vec<u8>| {
                (b.as_mut_ptr(), unsafe {
                    b.as_mut_ptr().offset(b.len() as isize)
                })
            }),
        };

        TestStreamBuffers {
            read_buffer: read_buffer,
            write_buffer: write_buffer,
            state: state,
        }
    }

    fn as_stream(&mut self, isatty: bool) -> TestStream {
        TestStream {
            isatty: isatty,
            attributes: TestTerminalAttributes::new(),
            state: &mut self.state,
        }
    }
}
