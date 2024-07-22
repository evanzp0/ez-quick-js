use std::{
    borrow::Cow,
    ffi::{c_char, CStr, CString},
};

use crate::{ffi::*, JsTag};

extern "C" {
    fn JS_ValueGetTag_real(v: JSValue) -> i32;
    fn JS_DupValue_real(ctx: *mut JSContext, v: JSValue);
    fn JS_DupValueRT_real(rt: *mut JSRuntime, v: JSValue);
    fn JS_FreeValue_real(ctx: *mut JSContext, v: JSValue);
    fn JS_FreeValueRT_real(rt: *mut JSRuntime, v: JSValue);
    fn JS_NewBool_real(ctx: *mut JSContext, v: bool) -> JSValue;
    fn JS_NewInt32_real(ctx: *mut JSContext, v: i32) -> JSValue;
    fn JS_NewFloat64_real(ctx: *mut JSContext, v: f64) -> JSValue;
    fn JS_VALUE_IS_NAN_real(v: JSValue) -> bool;
    fn JS_VALUE_GET_FLOAT64_real(v: JSValue) -> f64;
    fn JS_VALUE_GET_NORM_TAG_real(v: JSValue) -> ::std::os::raw::c_int;
    fn JS_IsNumber_real(v: JSValue) -> bool;
    fn JS_IsBigInt_real(ctx: *mut JSContext, v: JSValue) -> bool;
    fn JS_IsBigFloat_real(v: JSValue) -> bool;
    fn JS_IsBigDecimal_real(v: JSValue) -> bool;
    fn JS_IsBool_real(v: JSValue) -> bool;
    fn JS_IsNull_real(v: JSValue) -> bool;
    fn JS_IsUndefined_real(v: JSValue) -> bool;
    fn JS_IsException_real(v: JSValue) -> bool;
    fn JS_IsUninitialized_real(v: JSValue) -> bool;
    fn JS_IsString_real(v: JSValue) -> bool;
    fn JS_IsSymbol_real(v: JSValue) -> bool;
    fn JS_IsObject_real(v: JSValue) -> bool;
    fn JS_ToUint32_real(ctx: *mut JSContext, pres: u32, val: JSValue) -> u32;
    fn JS_SetProperty_real(
        ctx: *mut JSContext,
        this_obj: JSValue,
        prop: JSAtom,
        val: JSValue,
    ) -> ::std::os::raw::c_int;

    fn JS_Find_Atom_real(ctx: *mut JSContext, name: *const c_char) -> JSAtom ;
}

/// Increment the refcount of this value
pub unsafe fn js_dup_value(ctx: *mut JSContext, v: JSValue) {
    JS_DupValue_real(ctx, v);
}

/// Increment the refcount of this value
pub unsafe fn js_dup_value_rt(rt: *mut JSRuntime, v: JSValue) {
    JS_DupValueRT_real(rt, v);
}

/// Decrement the refcount of this value
pub unsafe fn js_free_value(ctx: *mut JSContext, v: JSValue) {
    JS_FreeValue_real(ctx, v);
}

/// Decrement the refcount of this value
pub unsafe fn js_free_value_rt(rt: *mut JSRuntime, v: JSValue) {
    JS_FreeValueRT_real(rt, v);
}

/// create a new boolean value
pub unsafe fn js_new_bool(ctx: *mut JSContext, v: bool) -> JSValue {
    JS_NewBool_real(ctx, v)
}

/// create a new int32 value
pub unsafe fn js_new_int32(ctx: *mut JSContext, v: i32) -> JSValue {
    JS_NewInt32_real(ctx, v)
}

/// create a new f64 value, please note that if the passed f64 fits in a i32 this will return a value with flag 0 (i32)
pub unsafe fn js_new_float64(ctx: *mut JSContext, v: f64) -> JSValue {
    JS_NewFloat64_real(ctx, v)
}

/// create a new String value
pub unsafe fn js_new_string(ctx: *mut JSContext, v: &str) -> JSValue {
    let mut val = format!("{v}\0");
    let val = val.as_bytes();
    JS_NewString(ctx, val as *const [u8] as *const c_char)
}

/// create a new Object value
pub unsafe fn js_new_object_with_proto(ctx: *mut JSContext, proto: Option<JSValue>) -> JSValue {
    if let Some(proto) = proto {
        JS_NewObjectProto(ctx, proto)
    } else {
        JS_NewObject(ctx)
    }
}

/// check if a JSValue is a NaN value
pub unsafe fn js_value_is_nan(v: JSValue) -> bool {
    JS_VALUE_IS_NAN_real(v)
}
/// get a f64 value from a JSValue
pub unsafe fn js_value_get_float64(v: JSValue) -> f64 {
    JS_VALUE_GET_FLOAT64_real(v)
}

/// same as JS_VALUE_GET_TAG, but return JS_TAG_FLOAT64 with NaN boxing
pub unsafe fn js_value_get_norm_tag(v: JSValue) -> ::std::os::raw::c_int {
    JS_VALUE_GET_NORM_TAG_real(v)
}

/// check if a JSValue is a Number
pub unsafe fn js_is_number(v: JSValue) -> bool {
    JS_IsNumber_real(v)
}

/// check if a JSValue is a BigInt
pub unsafe fn js_is_bigint(ctx: *mut JSContext, v: JSValue) -> bool {
    JS_IsBigInt_real(ctx, v)
}

/// check if a JSValue is a BigFloat
pub unsafe fn js_is_big_float(v: JSValue) -> bool {
    JS_IsBigFloat_real(v)
}

/// check if a JSValue is a BigDecimal
pub unsafe fn js_is_big_decimal(v: JSValue) -> bool {
    JS_IsBigDecimal_real(v)
}

/// check if a JSValue is a Boolean
pub unsafe fn js_is_bool(v: JSValue) -> bool {
    JS_IsBool_real(v)
}

/// check if a JSValue is null
pub unsafe fn js_is_null(v: JSValue) -> bool {
    JS_IsNull_real(v)
}

/// check if a JSValue is Undefined
pub unsafe fn js_is_undefined(v: JSValue) -> bool {
    JS_IsUndefined_real(v)
}

/// check if a JSValue is an Exception
pub unsafe fn js_is_exception(v: JSValue) -> bool {
    JS_IsException_real(v)
}

/// check if a JSValue is initialized
pub unsafe fn js_is_uninitialized(v: JSValue) -> bool {
    JS_IsUninitialized_real(v)
}

/// check if a JSValue is a String
pub unsafe fn js_is_string(v: JSValue) -> bool {
    JS_IsString_real(v)
}

/// check if a JSValue is a Symbol
pub unsafe fn js_is_symbol(v: JSValue) -> bool {
    JS_IsSymbol_real(v)
}

/// check if a JSValue is an Object
pub unsafe fn js_is_object(v: JSValue) -> bool {
    JS_IsObject_real(v)
}

/// set a property of an object identified by a JSAtom
pub unsafe fn js_set_property(
    ctx: *mut JSContext,
    this_obj: JSValue,
    prop: JSAtom,
    val: JSValue,
) -> ::std::os::raw::c_int {
    JS_SetProperty_real(ctx, this_obj, prop, val)
}

pub unsafe fn js_value_get_tag(v: JSValue) -> i32 {
    JS_ValueGetTag_real(v)
}

pub fn js_to_bool(ctx: *mut JSContext, val: JSValue) -> bool {
    unsafe { JS_ToBool(ctx, val) == 1 }
}

pub fn js_to_i32(ctx: *mut JSContext, val: JSValue) -> i32 {
    let mut rst = 0;
    unsafe { JS_ToInt32(ctx, &mut rst as *mut i32, val) };

    rst
}

pub fn js_to_f64(ctx: *mut JSContext, val: JSValue) -> f64 {
    match JsTag::from_c(&val) {
        JsTag::Int => unsafe { val.u.int32 as f64 },
        JsTag::Float64 => unsafe { val.u.float64 },
        JsTag::BigFloat => todo!(),
        JsTag::BigDecimal => todo!(),
        _other => {
            unreachable!()
        }
    }
}

pub fn js_to_string<'a>(ctx: *mut JSContext, val: JSValue) -> Cow<'a, str> {
    let mut len = 0_usize;
    let val = unsafe { JS_ToCStringLen2(ctx, &mut len as *mut usize, val, 0) };
    let val = unsafe { CStr::from_ptr(val) };

    val.to_string_lossy()
}

pub fn js_equal(ctx: *mut JSContext, one: &JSValue, other: &JSValue) -> bool {
    if one.tag != other.tag {
        return false;
    }

    match one.tag as i32 {
        crate::ffi::JS_TAG_INT => unsafe { one.u.int32 == other.u.int32 },
        crate::ffi::JS_TAG_BOOL => unsafe { one.u.int32 == other.u.int32 },
        crate::ffi::JS_TAG_NULL => unsafe { one.u.int32 == other.u.int32 },
        crate::ffi::JS_TAG_FLOAT64 => unsafe { one.u.float64 == other.u.float64 },
        crate::ffi::JS_TAG_STRING => unsafe {
            js_to_string(ctx, *one) == js_to_string(ctx, *other)
        },
        crate::ffi::JS_TAG_MODULE => todo!(),
        crate::ffi::JS_TAG_OBJECT => todo!(),
        crate::ffi::JS_TAG_SYMBOL => todo!(),
        crate::ffi::JS_TAG_BIG_FLOAT => todo!(),
        crate::ffi::JS_TAG_EXCEPTION => todo!(),
        crate::ffi::JS_TAG_UNDEFINED => todo!(),
        crate::ffi::JS_TAG_BIG_DECIMAL => todo!(),
        crate::ffi::JS_TAG_CATCH_OFFSET => todo!(),
        crate::ffi::JS_TAG_UNINITIALIZED => todo!(),
        crate::ffi::JS_TAG_FUNCTION_BYTECODE => todo!(),
        #[cfg(feature = "bigint")]
        ffi::JS_TAG_BIG_INT => JsTag::BigInt,
        _other => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, JsInteger, Runtime};
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn test_to_fn() {
        unsafe {
            let rt = Runtime::new(None);
            let ctx = &Context::new(rt);
            let js_int = JsInteger::new(ctx, 12);
            let js_val = js_int.to_value();
            let mut ret_val = 0_u32;
            let rst = js_to_i32(ctx.inner, js_val.inner);

            assert_eq!(12, rst);
        }
    }

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
