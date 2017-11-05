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

pub struct FnInstrumentation {
    call_count: Mutex<u64>,
}

impl FnInstrumentation {
    pub fn new() -> FnInstrumentation { FnInstrumentation { call_count: Mutex::new(0) } }

    pub fn record_call(&self) {
        let mut data = self.call_count.lock().unwrap();
        *data += 1;
    }

    pub fn get_call_count(&self) -> u64 { *self.call_count.lock().unwrap() }
}

#[cfg(test)]
mod test {

    use super::FnInstrumentation;
    use std::boxed::Box;

    #[test]
    fn test_fn_mut_instrumentation() {
        let instrumentation = FnInstrumentation::new();
        let mut function: Box<FnMut()> = Box::new(|| { instrumentation.record_call(); });

        assert!(instrumentation.get_call_count() == 0);
        function.as_mut()();
        assert!(instrumentation.get_call_count() == 1);
    }
}