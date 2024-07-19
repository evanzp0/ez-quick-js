use std::{
    borrow::Cow,
    ffi::{c_char, CStr},
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
    fn JS_NewCFunction_real(
        ctx: *mut JSContext,
        func: *mut JSCFunction,
        name: *const ::std::os::raw::c_char,
        length: ::std::os::raw::c_int,
    ) -> JSValue;
    fn JS_NewCFunctionMagic_real(
        ctx: *mut JSContext,
        func: *mut JSCFunctionMagic,
        name: *const ::std::os::raw::c_char,
        length: ::std::os::raw::c_int,
        cproto: JSCFunctionEnum,
        magic: ::std::os::raw::c_int,
    ) -> JSValue;
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
pub unsafe fn js_new_object(ctx: *mut JSContext, proto: Option<JSValue>) -> JSValue {
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

/// create a new Function based on a JSCFunction
pub unsafe fn js_new_cfunction(
    ctx: *mut JSContext,
    func: *mut JSCFunction,
    name: *const ::std::os::raw::c_char,
    length: ::std::os::raw::c_int,
) -> JSValue {
    JS_NewCFunction_real(ctx, func, name, length)
}

/// create a new Function based on a JSCFunction
pub unsafe fn js_new_cfunction_magic(
    ctx: *mut JSContext,
    func: *mut JSCFunctionMagic,
    name: *const ::std::os::raw::c_char,
    length: ::std::os::raw::c_int,
    cproto: JSCFunctionEnum,
    magic: ::std::os::raw::c_int,
) -> JSValue {
    JS_NewCFunctionMagic_real(ctx, func, name, length, cproto, magic)
}

pub unsafe fn js_value_get_tag(v: JSValue) -> i32 {
    JS_ValueGetTag_real(v)
}

/// get a u32 value from a JSValue
pub unsafe fn js_to_uint32(ctx: *mut JSContext, pres: u32, val: JSValue) -> u32 {
    JS_ToUint32_real(ctx, pres, val)
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
