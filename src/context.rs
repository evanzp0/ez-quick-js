use crate::{
    ffi::{js_new_object_with_proto, JSContext, JS_FreeContext, JS_FreeRuntime, JS_NewContext},
    function::{get_global_object, js_eval, new_object_with_proto},
    JsValue, Runtime,
};

pub struct Context<'a> {
    runtime: &'a Runtime,
    pub inner: *mut JSContext,
}

impl<'a> Context<'a> {
    pub fn new(runtime: &'a Runtime) -> Self {
        let inner = unsafe { JS_NewContext(runtime.inner) };
        if inner.is_null() {
            panic!("Context create failed");
        }

        Self { runtime, inner }
    }

    pub fn get_runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn new_global_object(&self) -> JsValue {
        get_global_object(self)
    }

    pub fn new_object(&self) -> JsValue {
        new_object_with_proto(self, None)
    }

    pub fn eval(
        &'a self,
        code: &str,
        file_name: &str,
        eval_flags: i32,
    ) -> Result<JsValue<'a>, crate::common::Error> {
        js_eval(self, code, file_name, eval_flags)
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.inner) }
    }
}
