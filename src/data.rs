// use crate::{ffi::{js_new_int32, JSValue, JS_ToInt32}, Context};
use crate::{
    ffi::{js_new_float64, js_new_int32, js_new_string, js_to_float64, js_to_i32},
    impl_from, impl_type_debug, impl_type_new, struct_type, impl_drop, impl_clone,
};

#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JsTag {
    // Used by C code as a marker.
    // Not relevant for bindings.
    // First = ffi::JS_TAG_FIRST,
    Int = crate::ffi::JS_TAG_INT,
    Bool = crate::ffi::JS_TAG_BOOL,
    Null = crate::ffi::JS_TAG_NULL,
    Module = crate::ffi::JS_TAG_MODULE,
    Object = crate::ffi::JS_TAG_OBJECT,
    String = crate::ffi::JS_TAG_STRING,
    Symbol = crate::ffi::JS_TAG_SYMBOL,
    #[cfg(feature = "bigint")]
    BigInt = crate::ffi::JS_TAG_BIG_INT,
    Float64 = crate::ffi::JS_TAG_FLOAT64,
    BigFloat = crate::ffi::JS_TAG_BIG_FLOAT,
    Exception = crate::ffi::JS_TAG_EXCEPTION,
    Undefined = crate::ffi::JS_TAG_UNDEFINED,
    BigDecimal = crate::ffi::JS_TAG_BIG_DECIMAL,
    CatchOffset = crate::ffi::JS_TAG_CATCH_OFFSET,
    Uninitialized = crate::ffi::JS_TAG_UNINITIALIZED,
    FunctionBytecode = crate::ffi::JS_TAG_FUNCTION_BYTECODE,
}

impl JsTag {
    #[inline]
    pub fn from_c(value: &crate::ffi::JSValue) -> JsTag {
        let inner = unsafe { crate::ffi::js_value_get_tag(*value) };
        match inner {
            crate::ffi::JS_TAG_INT => JsTag::Int,
            crate::ffi::JS_TAG_BOOL => JsTag::Bool,
            crate::ffi::JS_TAG_NULL => JsTag::Null,
            crate::ffi::JS_TAG_MODULE => JsTag::Module,
            crate::ffi::JS_TAG_OBJECT => JsTag::Object,
            crate::ffi::JS_TAG_STRING => JsTag::String,
            crate::ffi::JS_TAG_SYMBOL => JsTag::Symbol,
            crate::ffi::JS_TAG_FLOAT64 => JsTag::Float64,
            crate::ffi::JS_TAG_BIG_FLOAT => JsTag::BigFloat,
            crate::ffi::JS_TAG_EXCEPTION => JsTag::Exception,
            crate::ffi::JS_TAG_UNDEFINED => JsTag::Undefined,
            crate::ffi::JS_TAG_BIG_DECIMAL => JsTag::BigDecimal,
            crate::ffi::JS_TAG_CATCH_OFFSET => JsTag::CatchOffset,
            crate::ffi::JS_TAG_UNINITIALIZED => JsTag::Uninitialized,
            crate::ffi::JS_TAG_FUNCTION_BYTECODE => JsTag::FunctionBytecode,
            #[cfg(feature = "bigint")]
            ffi::JS_TAG_BIG_INT => JsTag::BigInt,
            _other => {
                unreachable!()
            }
        }
    }

    pub fn to_c(self) -> i32 {
        // TODO: figure out why this is needed
        // Just casting with `as` does not work correctly
        match self {
            JsTag::Int => crate::ffi::JS_TAG_INT,
            JsTag::Bool => crate::ffi::JS_TAG_BOOL,
            JsTag::Null => crate::ffi::JS_TAG_NULL,
            JsTag::Module => crate::ffi::JS_TAG_MODULE,
            JsTag::Object => crate::ffi::JS_TAG_OBJECT,
            JsTag::String => crate::ffi::JS_TAG_STRING,
            JsTag::Symbol => crate::ffi::JS_TAG_SYMBOL,
            JsTag::Float64 => crate::ffi::JS_TAG_FLOAT64,
            JsTag::BigFloat => crate::ffi::JS_TAG_BIG_FLOAT,
            JsTag::Exception => crate::ffi::JS_TAG_EXCEPTION,
            JsTag::Undefined => crate::ffi::JS_TAG_UNDEFINED,
            JsTag::BigDecimal => crate::ffi::JS_TAG_BIG_DECIMAL,
            JsTag::CatchOffset => crate::ffi::JS_TAG_CATCH_OFFSET,
            JsTag::Uninitialized => crate::ffi::JS_TAG_UNINITIALIZED,
            JsTag::FunctionBytecode => crate::ffi::JS_TAG_FUNCTION_BYTECODE,
            #[cfg(feature = "bigint")]
            JsTag::BigInt => crate::ffi::JS_TAG_FUNCTION_BYTECODE,
        }
    }

    /// Returns `true` if the js_tag is [`Undefined`].
    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    /// Returns `true` if the js_tag is [`Object`].
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object)
    }

    /// Returns `true` if the js_tag is [`Exception`].
    #[inline]
    pub fn is_exception(&self) -> bool {
        matches!(self, Self::Exception)
    }

    /// Returns `true` if the js_tag is [`Int`].
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int)
    }

    /// Returns `true` if the js_tag is [`Int`].
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Int | Self::Float64)
    }

    /// Returns `true` if the js_tag is [`Bool`].
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    /// Returns `true` if the js_tag is [`Null`].
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns `true` if the js_tag is [`Module`].
    #[inline]
    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module)
    }

    /// Returns `true` if the js_tag is [`String`].
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    /// Returns `true` if the js_tag is [`Symbol`].
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol)
    }

    /// Returns `true` if the js_tag is [`BigInt`].
    #[cfg(feature = "bigint")]
    #[inline]
    pub fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt)
    }

    /// Returns `true` if the js_tag is [`Float64`].
    #[inline]
    pub fn is_float64(&self) -> bool {
        matches!(self, Self::Float64)
    }

    /// Returns `true` if the js_tag is [`BigFloat`].
    #[inline]
    pub fn is_big_float(&self) -> bool {
        matches!(self, Self::BigFloat)
    }

    /// Returns `true` if the js_tag is [`BigDecimal`].
    #[inline]
    pub fn is_big_decimal(&self) -> bool {
        matches!(self, Self::BigDecimal)
    }
}

pub struct JsAtom<'a> {
    pub(crate) ctx: &'a crate::Context<'a>,
    pub(crate) inner: crate::ffi::JSAtom,
}

impl<'a> JsAtom<'a> {
    #[inline]
    pub fn new(ctx: &'a crate::Context, value: crate::ffi::JSAtom) -> Self {
        Self { ctx, inner: value }
    }
}

impl<'a> Drop for JsAtom<'a> {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::JS_FreeAtom(self.ctx.inner, self.inner);
        }
    }
}

impl<'a> Clone for JsAtom<'a> {
    fn clone(&self) -> Self {
        unsafe { crate::ffi::JS_DupAtom(self.ctx.inner, self.inner) };
        Self {
            ctx: self.ctx,
            inner: self.inner,
        }
    }
}

// pub struct Integer<'a> {
//     pub(crate) ctx: &'a Context<'a>,
//     pub(crate) inner: JSValue,
// }
// impl<'a> Integer<'a> {
//     pub fn new(ctx: &'a Context, v: i32) -> Self {
//         Self {
//             ctx,
//             inner: unsafe { js_new_int32(ctx.inner, v) },
//         }
//     }
// }
// impl<'a> Debug for Integer<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut f = f.debug_tuple("Integer");
//         if JsTag::from_c(&self.inner).is_int() {
//             let mut val = 0;
//             unsafe { JS_ToInt32(self.ctx.inner, &mut val, self.inner) };
//             f.field(&val);
//         } else {
//             f.field(&"unknown");
//         }
//         f.finish()
//     }
// }

struct_type!(JsInteger);
impl_type_debug!(JsInteger, is_int, crate::ffi::js_to_i32);
impl_type_new!(JsInteger, i32, crate::ffi::js_new_int32);
impl_drop!(JsInteger);
impl_clone!(JsInteger);
impl<'a> From<JsNumber<'a>> for JsInteger<'a>{
    fn from(value: JsNumber<'a>) -> Self {
        let JsNumber {ctx, inner: inner_val} = value;
        let inner = {
            let v = js_to_i32(ctx.inner, inner_val);
            unsafe { js_new_int32(ctx.inner, v) }
        };

        Self {
            ctx,
            inner
        }
    }
}

struct_type!(JsNumber);
impl_type_new!(JsNumber, f64, crate::ffi::js_new_float64);
impl_type_debug!(JsNumber, is_number, crate::ffi::js_to_float64);
impl_drop!(JsNumber);
impl_clone!(JsNumber);
impl_from!(JsInteger for JsNumber);

struct_type!(Boolean);
impl_type_new!(Boolean, bool, crate::ffi::js_new_bool);
impl_type_debug!(Boolean, is_bool, crate::ffi::js_to_bool);
impl_drop!(Boolean);
impl_clone!(Boolean);

struct_type!(JsString);
impl_type_new!(JsString, &str, crate::ffi::js_new_string);
impl_type_debug!(JsString, is_string, crate::ffi::js_to_string);
impl_drop!(JsString);
impl_clone!(JsString);

struct_type!(JsValue);
impl<'a> JsValue<'a> {
    pub fn new(ctx: &'a crate::Context, value: crate::ffi::JSValue) -> Self {
        Self { ctx, inner: value }
    }
}
impl<'a> std::fmt::Debug for JsValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("JsValue").field(&"..").finish()
    }
}
impl_drop!(JsValue);
impl_clone!(JsValue);

/////////////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! struct_type {
    ($type:ident) => {
        pub struct $type<'a> {
            pub(crate) ctx: &'a crate::Context<'a>,
            pub inner: crate::ffi::JSValue,
        }
    };
}

#[macro_export]
macro_rules! impl_type_new {
    ($type:ident, $val_type:ty, $js_ctor:path) => {
        impl<'a> $type<'a> {
            pub fn new(ctx: &'a crate::Context, v: $val_type) -> Self {
                Self {
                    ctx,
                    inner: unsafe { $js_ctor(ctx.inner, v) },
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_type_debug {
    ($type:ident, $fn:ident, $converter:path) => {
        impl<'a> std::fmt::Debug for $type<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_tuple(stringify!($type));
                // f.debug_struct("Integer").field("ctx", &self.ctx).field("inner", &self.inner).finish()
                if JsTag::from_c(&self.inner).$fn() {
                    let val = unsafe { $converter(self.ctx.inner, self.inner) };
                    f.field(&val);
                } else {
                    f.field(&"unknown");
                }

                f.finish()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from {
    { $source:ident for $type:ident } => {
      impl<'s> From<$source<'s>> for $type<'s> {
        fn from(l: $source<'s>) -> Self {
          unsafe { std::mem::transmute(l) }
        }
      }
    };
}

#[macro_export]
macro_rules! impl_drop {
    { $type:ident } => {
        impl<'a> Drop for $type<'a> {
            fn drop(&mut self) {
                unsafe {
                    crate::ffi::js_free_value(self.ctx.inner, self.inner);
                }
            }
        }        
    };
}

#[macro_export]
macro_rules! impl_clone {
    { $type:ident } => {
        impl<'a> Clone for $type<'a> {
            fn clone(&self) -> Self {
                unsafe { crate::ffi::js_dup_value(self.ctx.inner, self.inner) };
                Self {
                    ctx: self.ctx,
                    inner: self.inner,
                }
            }
        }      
    };
}

