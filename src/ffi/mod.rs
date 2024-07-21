mod static_functions;
mod bindings;

pub(crate) use bindings::*;
pub(crate) use static_functions::*;

pub use bindings::{JSCFunction, JSCFunctionMagic, JSCFunctionData};

pub use bindings::{
    JS_EVAL_TYPE_GLOBAL,
    JS_EVAL_TYPE_MODULE,
    JS_EVAL_TYPE_DIRECT,
    JS_EVAL_TYPE_INDIRECT,
    JS_EVAL_TYPE_MASK,
    JS_EVAL_FLAG_STRICT,
    JS_EVAL_FLAG_STRIP,
    JS_EVAL_FLAG_COMPILE_ONLY,
    JS_EVAL_FLAG_BACKTRACE_BARRIER,
    JS_EVAL_FLAG_ASYNC,
};