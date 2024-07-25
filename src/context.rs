use crate::{
    common::Error,
    ffi::{
        JSCFunction, JSContext, JSModuleInitFunc, JS_FreeContext, JS_FreeRuntime, JS_GetRuntime,
        JS_NewContext, JS_NewObjectWithProto,
    },
    function::{
        get_global_object, js_eval, new_atom, new_c_function, new_c_module, new_object_with_proto,
    },
    CFunctionInner, JsAtom, JsBoolean, JsInteger, JsModuleDef, JsNumber, JsString, JsValue,
    Runtime, JS_NULL, JS_UNDEFINED,
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

    pub fn new_module(
        &self,
        module_name: &str,
        module_init_func: JSModuleInitFunc,
    ) -> Result<JsModuleDef, Error> {
        new_c_module(self, module_name, module_init_func)
    }

    // pub fn from_raw(js_ctx: *mut JSContext) -> Self {
    //     let runtime = {
    //         let rt = unsafe { JS_GetRuntime(js_ctx) };
    //         Runtime::from_raw(rt)
    //     };

    //     Self {
    //         runtime,
    //         inner: js_ctx,
    //     }
    // }

    pub unsafe fn forget(self) -> *mut JSContext {
        let v = self.inner;
        std::mem::forget(self);
        v
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

    pub fn new_prototype(&self, proto: JsValue) -> Result<JsValue, Error> {
        new_object_with_proto(self, Some(proto))
    }

    pub fn new_atom(&self, name: &str) -> Result<JsAtom, Error> {
        new_atom(self, name)
    }

    pub fn eval(
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

    pub fn get_undefined(&self) -> JsValue {
        JsValue::new(self, JS_UNDEFINED)
    }

    pub fn get_cfunction(
        &self,
        c_func: CFunctionInner,
        name: &str,
        arg_count: i32,
    ) -> Result<JsValue, Error> {
        new_c_function(self, Some(c_func), name, arg_count)
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.inner) }
    }
}
