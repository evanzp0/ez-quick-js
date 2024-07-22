use crate::{
    common::Error,
    ffi::{
        js_new_object_with_proto, JSCFunction, JSContext, JS_FreeContext, JS_FreeRuntime,
        JS_GetRuntime, JS_NewContext,
    },
    function::{get_global_object, js_eval, new_atom, new_cfunction, new_object_with_proto},
    CFunctionInner, JsAtom, JsBoolean, JsInteger, JsNumber, JsString, JsValue, Runtime,
};

pub struct Context {
    runtime: Runtime,
    pub inner: *mut JSContext,
}

impl Context {
    pub fn new(runtime: Runtime) -> Self {
        let inner = unsafe { JS_NewContext(runtime.inner) };
        if inner.is_null() {
            panic!("Context create failed");
        }

        Self { runtime, inner }
    }

    pub fn from_raw(js_ctx: *mut JSContext) -> Self {
        let runtime = {
            let rt = unsafe { JS_GetRuntime(js_ctx) };
            Runtime::from_raw(rt)
        };

        Self {
            runtime,
            inner: js_ctx,
        }
    }

    pub fn get_runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn get_global_object(&self) -> JsValue {
        get_global_object(self)
    }

    pub fn new_object(&self) -> Result<JsValue, Error> {
        new_object_with_proto(self, None)
    }

    pub fn new_atom(&self, name: &str) -> Result<JsAtom, Error> {
        new_atom(self, name)
    }

    pub fn eval<'a>(
        &'a self,
        code: &str,
        file_name: &str,
        eval_flags: i32,
    ) -> Result<JsValue<'a>, crate::common::Error> {
        js_eval(self, code, file_name, eval_flags)
    }

    pub fn get_number(&self, val: f64) -> JsValue {
        JsNumber::new(self, val).into()
    }

    pub fn get_int(&self, val: i32) -> JsValue {
        JsInteger::new(self, val).into()
    }

    pub fn get_string(&self, val: &str) -> JsValue {
        JsString::new(self, val).into()
    }

    pub fn get_bool(&self, val: bool) -> JsValue {
        JsBoolean::new(self, val).into()
    }

    pub fn get_cfunction(
        &self,
        c_func: CFunctionInner,
        name: &str,
        arg_count: i32,
    ) -> Result<JsValue, Error> {
        new_cfunction(self, Some(c_func), name, arg_count)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.inner) }
    }
}
