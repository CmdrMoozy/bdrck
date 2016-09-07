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
    use std::boxed::Box;

    use super::FnInstrumentation;

    #[test]
    fn test_fn_mut_instrumentation() {
        let instrumentation = FnInstrumentation::new();
        let mut function: Box<FnMut()> = Box::new(|| {
            instrumentation.record_call();
        });

        assert!(instrumentation.get_call_count() == 0);
        function.as_mut()();
        assert!(instrumentation.get_call_count() == 1);
    }
}
