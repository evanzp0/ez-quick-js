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
pub unsafe fn JS_DupValue(ctx: *mut JSContext, v: JSValue) {
    JS_DupValue_real(ctx, v);
}

/// Increment the refcount of this value
pub unsafe fn JS_DupValueRT(rt: *mut JSRuntime, v: JSValue) {
    JS_DupValueRT_real(rt, v);
}

/// Decrement the refcount of this value
pub unsafe fn JS_FreeValue(ctx: *mut JSContext, v: JSValue) {
    JS_FreeValue_real(ctx, v);
}

/// Decrement the refcount of this value
pub unsafe fn JS_FreeValueRT(rt: *mut JSRuntime, v: JSValue) {
    JS_FreeValueRT_real(rt, v);
}

/// create a new boolean value
pub unsafe fn JS_NewBool(ctx: *mut JSContext, v: bool) -> JSValue {
    JS_NewBool_real(ctx, v)
}

/// create a new int32 value
pub unsafe fn JS_NewInt32(ctx: *mut JSContext, v: i32) -> JSValue {
    JS_NewInt32_real(ctx, v)
}

/// create a new f64 value, please note that if the passed f64 fits in a i32 this will return a value with flag 0 (i32)
pub unsafe fn JS_NewFloat64(ctx: *mut JSContext, v: f64) -> JSValue {
    JS_NewFloat64_real(ctx, v)
}

/// create a new String value
pub unsafe fn JS_NewStr(ctx: *mut JSContext, v: &str) -> JSValue {
    let mut val = format!("{v}\0");
    let val = val.as_bytes();
    JS_NewString(ctx, val as *const [u8] as *const c_char)
}

/// create a new Object value
pub unsafe fn JS_NewObjectWithProto(ctx: *mut JSContext, proto: Option<JSValue>) -> JSValue {
    if let Some(proto) = proto {
        JS_NewObjectProto(ctx, proto)
    } else {
        JS_NewObject(ctx)
    }
}

/// check if a JSValue is a NaN value
pub unsafe fn JS_VALUE_IS_NAN(v: JSValue) -> bool {
    JS_VALUE_IS_NAN_real(v)
}
/// get a f64 value from a JSValue
pub unsafe fn JS_VALUE_GET_FLOAT64(v: JSValue) -> f64 {
    JS_VALUE_GET_FLOAT64_real(v)
}

/// same as JS_VALUE_GET_TAG, but return JS_TAG_FLOAT64 with NaN boxing
pub unsafe fn JS_VALUE_GET_NORM_TAG(v: JSValue) -> ::std::os::raw::c_int {
    JS_VALUE_GET_NORM_TAG_real(v)
}

/// check if a JSValue is a Number
pub unsafe fn JS_IsNumber(v: JSValue) -> bool {
    JS_IsNumber_real(v)
}

/// check if a JSValue is a BigInt
pub unsafe fn JS_IsBigInt(ctx: *mut JSContext, v: JSValue) -> bool {
    JS_IsBigInt_real(ctx, v)
}

/// check if a JSValue is a BigFloat
pub unsafe fn JS_IsBigFloat(v: JSValue) -> bool {
    JS_IsBigFloat_real(v)
}

/// check if a JSValue is a BigDecimal
pub unsafe fn JS_IsBigDecimal(v: JSValue) -> bool {
    JS_IsBigDecimal_real(v)
}

/// check if a JSValue is a Boolean
pub unsafe fn JS_IsBool(v: JSValue) -> bool {
    JS_IsBool_real(v)
}

/// check if a JSValue is null
pub unsafe fn JS_IsNull(v: JSValue) -> bool {
    JS_IsNull_real(v)
}

/// check if a JSValue is Undefined
pub unsafe fn JS_IsUndefined(v: JSValue) -> bool {
    JS_IsUndefined_real(v)
}

/// check if a JSValue is an Exception
pub unsafe fn JS_IsException(v: JSValue) -> bool {
    JS_IsException_real(v)
}

/// check if a JSValue is initialized
pub unsafe fn JS_IsUninitialized(v: JSValue) -> bool {
    JS_IsUninitialized_real(v)
}

/// check if a JSValue is a String
pub unsafe fn JS_IsString(v: JSValue) -> bool {
    JS_IsString_real(v)
}

/// check if a JSValue is a Symbol
pub unsafe fn JS_IsSymbol(v: JSValue) -> bool {
    JS_IsSymbol_real(v)
}

/// check if a JSValue is an Object
pub unsafe fn JS_IsObject(v: JSValue) -> bool {
    JS_IsObject_real(v)
}

/// set a property of an object identified by a JSAtom
pub unsafe fn JS_SetProperty(
    ctx: *mut JSContext,
    this_obj: JSValue,
    prop: JSAtom,
    val: JSValue,
) -> ::std::os::raw::c_int {
    JS_SetProperty_real(ctx, this_obj, prop, val)
}

pub unsafe fn JS_ValueGetTag(v: JSValue) -> i32 {
    JS_ValueGetTag_real(v)
}

pub fn JS_ToBoolean(ctx: *mut JSContext, val: JSValue) -> bool {
    unsafe { JS_ToBool(ctx, val) == 1 }
}

pub fn JS_ToI32(ctx: *mut JSContext, val: JSValue) -> i32 {
    let mut rst = 0;
    unsafe { JS_ToInt32(ctx, &mut rst as *mut i32, val) };

    rst
}

pub fn JS_ToF64(ctx: *mut JSContext, val: JSValue) -> f64 {
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

pub fn JS_ToStr<'a>(ctx: *mut JSContext, val: JSValue) -> Cow<'a, str> {
    let mut len = 0_usize;
    let val = unsafe { JS_ToCStringLen2(ctx, &mut len as *mut usize, val, 0) };
    let val = unsafe { CStr::from_ptr(val) };

    val.to_string_lossy()
}

pub fn JS_Equal(ctx: *mut JSContext, one: &JSValue, other: &JSValue) -> bool {
    if one.tag != other.tag {
        return false;
    }

    match one.tag as i32 {
        crate::ffi::JS_TAG_INT => unsafe { one.u.int32 == other.u.int32 },
        crate::ffi::JS_TAG_BOOL => unsafe { one.u.int32 == other.u.int32 },
        crate::ffi::JS_TAG_NULL => unsafe { one.u.int32 == other.u.int32 },
        crate::ffi::JS_TAG_FLOAT64 => unsafe { one.u.float64 == other.u.float64 },
        crate::ffi::JS_TAG_STRING => unsafe {
            JS_ToStr(ctx, *one) == JS_ToStr(ctx, *other)
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
            let rt = JS_NewRuntime();
            let ctx = JS_NewContext(rt);
            let js_int = JS_NewInt32(ctx, 12);
            let val = JS_ToI32(ctx, js_int);
            assert_eq!(12, val);
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

            JS_DupValue(ctx, value);
            JS_FreeValue(ctx, value);

            let ival = JS_NewInt32(ctx, 12);
            assert_eq!(ival.tag, 0);
            let fval = JS_NewFloat64(ctx, f64::MAX);
            assert_eq!(fval.tag, 7);
            let bval = JS_NewBool(ctx, true);
            assert_eq!(bval.tag, 1);
        }
    }
}
