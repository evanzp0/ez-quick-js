use crate::ffi::{JSRuntime, JS_FreeRuntime, JS_NewRuntime, JS_SetMemoryLimit};

pub struct Runtime {
    pub(crate) inner: *mut JSRuntime,
}

impl Runtime {
    pub fn new(memory_limit: Option<usize>) -> Self {
        let inner = unsafe { JS_NewRuntime() };
        if inner.is_null() {
            panic!("Runtime create failed");
        }
        // Configure memory limit if specified.
        if let Some(limit) = memory_limit {
            unsafe {
                JS_SetMemoryLimit(inner, limit);
            }
        }

        Self { inner }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { JS_FreeRuntime(self.inner) }
    }
}
