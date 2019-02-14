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

use std::io::{self, Write};
use std::sync::{Arc, Mutex, MutexGuard};

/// A LogOutputFactory is a function which can be called repeatedly, returning
/// a Write implementor each time which can be used for log output.
///
/// This is necessary because upstream's Log::log function is &self, whereas all
/// of the interesting member functions on Write are &mut self.
pub type LogOutputFactory = Box<dyn Fn() -> Box<dyn Write> + Send + Sync>;

/// This struct is a simple adapter which is composed of a single Write
/// implementor. The idea is that it is guaranteed to implement Send + Sync, as
/// well as Clone, so it allows arbitrary Write implementors to be used with
/// LogOutputFactory.
///
/// Note that this does involve a Mutex, so in general there is a performance
/// penalty to using this.
pub struct SyncWriteAdapter<T: Write> {
    writer: Arc<Mutex<T>>,
}

impl<T: Write + Send + 'static> SyncWriteAdapter<T> {
    /// Create a new SyncWriteAdapter which wraps the given Write implementor.
    pub fn new(writer: T) -> Self {
        SyncWriteAdapter {
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    /// Acquire a lock on the internal Write implementor, which allows for e.g.
    /// writing output to it without having a mutable reference to this adapter.
    pub fn lock(&self) -> MutexGuard<T> {
        self.writer.lock().unwrap()
    }
}

impl<T: Write + Send + 'static> Write for SyncWriteAdapter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut lock = self.writer.lock().unwrap();
        lock.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut lock = self.writer.lock().unwrap();
        lock.flush()
    }
}

// Clone cannot be derived in this case, due to something like:
// https://github.com/rust-lang/rust/issues/26925
impl<T: Write + Send + 'static> Clone for SyncWriteAdapter<T> {
    fn clone(&self) -> Self {
        SyncWriteAdapter {
            writer: self.writer.clone(),
        }
    }
}

/// This is a convenience function, which makes it trivial to construct a new
/// LogOutputFactory from an arbitrary Write implementor, using the existing
/// structures / functions defined elsewhere in this module.
pub fn new_log_output_factory<T: Write + Send + 'static>(writer: T) -> LogOutputFactory {
    let writer: SyncWriteAdapter<T> = SyncWriteAdapter::new(writer);
    Box::new(move || Box::new(writer.clone()))
}
