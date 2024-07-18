use crate::{
    ffi::{JSContext, JS_FreeContext, JS_FreeRuntime, JS_NewContext},
    Runtime,
};

pub struct Context<'a> {
    runtime: &'a Runtime,
    pub(crate) inner: *mut JSContext,
}

impl<'a> Context<'a> {
    pub fn new(runtime: &'a Runtime) -> Self {
        let inner = unsafe { JS_NewContext(runtime.inner) };
        if inner.is_null() {
            panic!("Context create failed");
        }

        Self { runtime, inner }
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.inner) }
    }
}
