//! FFI Bindings for [quickjs](https://bellard.org/quickjs/),
//! a Javascript engine.
//! See the [quickjs](https://crates.io/crates/quickjs) crate for a high-level
//! wrapper.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(unused)]

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
// include!("static-functions.rs");

mod ffi;
pub(crate) use ffi::*;

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    // Small sanity test that starts the runtime and evaluates code.
    #[test]
    fn test_eval() {
        unsafe {
            let rt = JS_NewRuntime();
            let ctx = JS_NewContext(rt);

            let code_str = "1 + 1\0";
            let code = CStr::from_bytes_with_nul(code_str.as_bytes()).unwrap();
            let script = CStr::from_bytes_with_nul("script\0".as_bytes()).unwrap();

            let value = JS_Eval(
                ctx,
                code.as_ptr(),
                code_str.len() - 1,
                script.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            );
            assert_eq!(value.tag, 0);
            assert_eq!(value.u.int32, 2);

            js_dup_value(ctx, value);
            js_free_value(ctx, value);

            let ival = js_new_int32(ctx, 12);
            assert_eq!(ival.tag, 0);
            let fval = js_new_float64(ctx, f64::MAX);
            assert_eq!(fval.tag, 7);
            let bval = js_new_bool(ctx, true);
            assert_eq!(bval.tag, 1);
        }
    }
}