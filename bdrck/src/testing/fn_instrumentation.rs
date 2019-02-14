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

use std::sync::Mutex;

/// This is a structure which contains the state relating to instrumenting a
/// function. The idea is that you would mutate it via its member functions to
/// e.g. record information about a function call. Because it has internal
/// synchronization, this can be done without retaining a mutable reference.
pub struct FnInstrumentation {
    call_count: Mutex<u64>,
}

impl FnInstrumentation {
    /// Construct a new instrumentation state, with default-initialized values.
    pub fn new() -> FnInstrumentation {
        FnInstrumentation {
            call_count: Mutex::new(0),
        }
    }

    /// Record that the function being instrumented was called by incrementing a
    /// counter.
    pub fn record_call(&self) {
        let mut data = self.call_count.lock().unwrap();
        *data += 1;
    }

    /// Return the current number of calls recorded.
    pub fn get_call_count(&self) -> u64 {
        *self.call_count.lock().unwrap()
    }
}
