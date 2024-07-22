use crate::{
    common::{make_cstring, Error},
    ffi::{
        js_dup_value, js_free_value, js_new_float64, js_new_int32, js_new_string, js_to_f64,
        js_to_i32, JSContext, JSRefCountHeader, JSValue, JS_ATOM_NULL,
    },
    function::{get_last_exception, run_compiled_function, to_bytecode},
};

macro_rules! struct_type {
    ($type:ident) => {
        pub struct $type<'a> {
            pub(crate) ctx: &'a crate::Context,
            pub(crate) inner: JSValue,
        }
    };
}

macro_rules! impl_type_common_fn {
    ($type:ident, $val_type:ty, $js_ctor:path) => {
        impl<'a> $type<'a> {
            pub fn new(ctx: &'a crate::Context, v: $val_type) -> Self {
                Self {
                    ctx,
                    inner: unsafe { $js_ctor(ctx.inner, v) },
                }
            }

            to_value_fn!();
            raw_value_fn!();
            context_fn!();
            tag_fn!();
            forget_fn!();
        }
    };
}

macro_rules! impl_type_debug {
    ($type:ident, $fn:ident, $converter:path) => {
        impl<'a> std::fmt::Debug for $type<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_tuple(stringify!($type));
                // f.debug_struct("Integer").field("ctx", &self.ctx).field("inner", &self.inner).finish()
                if self.tag().$fn() {
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

macro_rules! impl_deref {
    { $target:ident for $type:ident } => {
        impl Deref for $type {
        type Target = $target;
        fn deref(&self) -> &Self::Target {
            unsafe { &*(self as *const _ as *const Self::Target) }
        }
        }
    };
}

macro_rules! impl_from {
    { $source:ident for $type:ident } => {
        impl<'s> From<$source<'s>> for $type<'s> {
            fn from(l: $source<'s>) -> Self {
                unsafe { std::mem::transmute(l) }
            }
        }
    };
}

macro_rules! impl_try_from {
    { $source:ident for $target:ident if $value:ident => $check:expr } => {
        impl<'s> TryFrom<$source<'s>> for $target<'s> {
            type Error = crate::common::Error;
            fn try_from(l: $source<'s>) -> Result<Self, Self::Error> {
                // Not dead: `cast()` is sometimes used in the $check expression.
                #[allow(dead_code)]
                fn cast(l: $source) -> $target {
                    unsafe { std::mem::transmute::<$source, $target>(l) }
                }
                match l {
                    $value if $check => Ok(unsafe {
                        std::mem::transmute::<$source<'s>, $target<'s>>($value)
                    }),
                    _ => Err(crate::common::Error::bad_type::<$source, $target>("TryFrom"))
                }
            }
        }
    };
}

macro_rules! is_fn {
    { $fn:ident } => {
        pub fn $fn(&self) -> bool {
            self.tag().$fn()
        }
    };
}

macro_rules! to_fn {
    { $fn:ident, $type:ident, $tag:path, $is_fn:ident } => {
        pub fn $fn(self) -> Result<$type<'a>, crate::common::Error> {
            if !self.tag().$is_fn() {
                Err(crate::common::Error::BadType(format!("Need {:?} but get {:?}", $tag, self.tag())))?
            }

            self.try_into()
        }
    };
}

macro_rules! raw_value_fn {
    {} => {
        pub fn raw_value(&self) -> &JSValue {
            &self.inner
        }
    };
}

macro_rules! context_fn {
    {} => {
        pub fn context(&self) -> &crate::Context {
            self.ctx
        }
    };
}

macro_rules! tag_fn {
    {} => {
        pub fn tag(&self) -> JsTag {
            JsTag::from_c(self.raw_value())
        }
    };
}

macro_rules! forget_fn {
    {} => {
        pub unsafe fn forget(self) -> JSValue {
            let v = self.inner;
            std::mem::forget(self);
            v
        }
    };
}

macro_rules! to_value_fn {
    {} => {
        pub fn to_value(self) -> JsValue<'a> {
            self.into()
        }
    };
}

macro_rules! impl_value_fn {
    {$type:ident, $fn:ident, $ret_type:path} => {
        impl<'a> $type<'a> {
            pub fn value(&self) -> $ret_type {
                crate::ffi::$fn(self.ctx.inner, self.inner)
            }
        }
    };
}

macro_rules! impl_eq {
    { for $type:ident } => {
        impl<'a> Eq for $type<'a> {}
    };
}

macro_rules! impl_partial_eq {
    { $rhs:ident for $type:ident } => {
        impl<'s> PartialEq<$rhs<'s>> for $type<'s> {
            fn eq(&self, other: &$rhs) -> bool {
                let a = self.raw_value();
                let b = other.raw_value();
                unsafe { crate::ffi::js_equal(self.context().inner, a, b) }
            }
        }
    };
}

//////////////////////////////////////////////////////////

pub type CFunctionInner = unsafe extern "C" fn(
    ctx: *mut JSContext,
    this_val: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue;

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
    pub fn from_c(value: &JSValue) -> JsTag {
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

    /// Check if this value is a bytecode compiled function.
    #[inline]
    pub fn is_compiled_function(&self) -> bool {
        matches!(self, Self::FunctionBytecode)
    }
}

pub struct JsAtom<'a> {
    pub(crate) ctx: &'a crate::Context,
    pub(crate) inner: crate::ffi::JSAtom,
}

impl<'a> JsAtom<'a> {
    #[inline]
    pub fn new(ctx: &'a crate::Context, value: crate::ffi::JSAtom) -> Self {
        Self { ctx, inner: value }
    }

    pub fn is_exception(&self) -> bool {
        self.inner == JS_ATOM_NULL
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

//////////////////////////////////////////////////////////////

struct_type!(JsInteger);
impl_type_debug!(JsInteger, is_int, crate::ffi::js_to_i32);
impl_type_common_fn!(JsInteger, i32, crate::ffi::js_new_int32);
impl_drop!(JsInteger);
impl_clone!(JsInteger);
impl_try_from!(JsValue for JsInteger if v => v.is_int());
impl<'a> From<JsNumber<'a>> for JsInteger<'a> {
    fn from(value: JsNumber<'a>) -> Self {
        let JsNumber {
            ctx,
            inner: inner_val,
        } = value;
        let inner = {
            let v = js_to_i32(ctx.inner, inner_val);
            unsafe { js_new_int32(ctx.inner, v) }
        };

        Self { ctx, inner }
    }
}
impl_eq!(for JsInteger);
impl_partial_eq!(JsInteger for JsInteger);
impl_value_fn!(JsInteger, js_to_i32, i32);

struct_type!(JsNumber);
impl_type_common_fn!(JsNumber, f64, crate::ffi::js_new_float64);
impl_type_debug!(JsNumber, is_number, crate::ffi::js_to_f64);
impl_drop!(JsNumber);
impl_clone!(JsNumber);
impl_from!(JsInteger for JsNumber);
impl_try_from!(JsValue for JsNumber if v => v.is_number());
impl_eq!(for JsNumber);
impl_partial_eq!(JsNumber for JsNumber);
impl_value_fn!(JsNumber, js_to_f64, f64);

struct_type!(JsBoolean);
impl_type_common_fn!(JsBoolean, bool, crate::ffi::js_new_bool);
impl_type_debug!(JsBoolean, is_bool, crate::ffi::js_to_bool);
impl_drop!(JsBoolean);
impl_clone!(JsBoolean);
impl_try_from!(JsValue for JsBoolean if v => v.is_bool());
impl_eq!(for JsBoolean);
impl_partial_eq!(JsBoolean for JsBoolean);
impl_value_fn!(JsBoolean, js_to_bool, bool);

struct_type!(JsString);
impl_type_common_fn!(JsString, &str, crate::ffi::js_new_string);
impl_type_debug!(JsString, is_string, crate::ffi::js_to_string);
impl_drop!(JsString);
impl_clone!(JsString);
impl_try_from!(JsValue for JsString if v => v.is_string());
impl_eq!(for JsString);
impl_partial_eq!(JsString for JsString);
impl_value_fn!(JsString, js_to_string, std::borrow::Cow<'_, str>);

struct_type!(JsValue);
impl<'a> JsValue<'a> {
    pub fn new(ctx: &'a crate::Context, value: JSValue) -> Self {
        Self { ctx, inner: value }
    }

    /// Check if this value is a Javascript function.
    pub fn is_function(&self) -> bool {
        unsafe { crate::ffi::JS_IsFunction(self.ctx.inner, self.inner) == 1 }
    }

    pub fn to_function(self) -> Result<JsFunction<'a>, Error> {
        let js_fn: JsFunction = self.try_into()?;
        Ok(js_fn)
    }

    pub fn is_array(&self) -> bool {
        unsafe { crate::ffi::JS_IsArray(self.ctx.inner, self.inner) == 1 }
    }

    pub fn borrow_value(&self) -> &JSValue {
        &self.inner
    }

    pub(crate) fn increment_ref_count(&self) {
        if self.inner.tag < 0 {
            unsafe { js_dup_value(self.ctx.inner, self.inner) }
        }
    }

    pub(crate) fn decrement_ref_count(&self) {
        if self.inner.tag < 0 {
            unsafe { js_free_value(self.ctx.inner, self.inner) }
        }
    }

    pub fn get_ref_count(&self) -> i32 {
        if self.inner.tag < 0 {
            // This transmute is OK since if tag < 0, the union will be a refcount
            // pointer.
            let ptr = unsafe { self.inner.u.ptr as *mut JSRefCountHeader };
            let pref: &mut JSRefCountHeader = &mut unsafe { *ptr };
            pref.ref_count
        } else {
            -1
        }
    }

    /// borrow the value but first increment the refcount, this is useful for when the value is returned or passed to functions
    pub fn dup_value(&self) -> JSValue {
        self.increment_ref_count();
        self.inner
    }

    pub fn set_property(&self, prop_name: &str, prop_value: JsValue) -> Result<(), Error> {
        let p_name = make_cstring(prop_name)?;

        let val = unsafe {
            crate::ffi::JS_SetPropertyStr(
                self.ctx.inner,
                self.inner,
                p_name.as_ptr(),
                prop_value.dup_value(),
            )
        };

        if val == -1 {
            Err(Error::GeneralError(format!(
                "Set property '{}' for object is failed",
                prop_name
            )))?;
        }

        // unsafe { prop_value.forget() };

        Ok(())
    }

    pub fn get_property(&self, prop_name: &str) -> Option<JsValue> {
        let p_name = {
            if let Ok(val) = make_cstring(prop_name) {
                val
            } else {
                return None;
            }
        };

        let val =
            unsafe { crate::ffi::JS_GetPropertyStr(self.ctx.inner, self.inner, p_name.as_ptr()) };

        let val = JsValue::new(self.ctx, val);

        if val.is_exception() || val.is_undefined() {
            None
        } else {
            Some(val)
        }
    }

    pub fn define_property(
        &self,
        prop_name: &str,
        prop_value: JsValue,
        flags: i32,
    ) -> Result<(), Error> {
        let prop_name = self.ctx.new_atom(prop_name)?;
        crate::function::define_property(self.ctx, self, prop_name, prop_value, flags)
    }

    is_fn!(is_undefined);
    is_fn!(is_object);
    is_fn!(is_exception);
    is_fn!(is_int);
    is_fn!(is_number);
    is_fn!(is_bool);
    is_fn!(is_null);
    is_fn!(is_module);
    is_fn!(is_string);
    is_fn!(is_symbol);
    is_fn!(is_float64);
    is_fn!(is_big_float);
    is_fn!(is_big_decimal);
    is_fn!(is_compiled_function);

    to_fn!(to_int, JsInteger, JsTag::Int, is_int);
    to_fn!(to_number, JsNumber, JsTag::Float64, is_number);
    to_fn!(to_bool, JsBoolean, JsTag::Bool, is_bool);
    to_fn!(to_string, JsString, JsTag::String, is_string);
    to_fn!(to_object, JsObject, JsTag::Object, is_object);
    to_fn!(to_module, JsModule, JsTag::Module, is_module);
    to_fn!(
        to_compiled_function,
        JsCompiledFunction,
        JsTag::FunctionBytecode,
        is_compiled_function
    );

    raw_value_fn!();
    context_fn!();
    tag_fn!();
    forget_fn!();
}
impl<'a> std::fmt::Debug for JsValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("JsValue")
            .field(&self.tag())
            .field(&"...")
            .finish()
    }
}
impl_drop!(JsValue);
impl_clone!(JsValue);
impl_from!(JsInteger for JsValue);
impl_from!(JsNumber for JsValue);
impl_from!(JsBoolean for JsValue);
impl_from!(JsString for JsValue);
impl_from!(JsObject for JsValue);
impl_from!(JsFunction for JsValue);
impl_from!(JsCompiledFunction for JsValue);
impl_from!(JsModule for JsValue);
impl_from!(JsArray for JsValue);

struct_type!(JsArray);
impl<'a> JsArray<'a> {
    pub fn new(ctx: &'a crate::Context, value: JSValue) -> Self {
        let is_array = unsafe { crate::ffi::JS_IsArray(ctx.inner, value) == 1 };

        if is_array {
            Self { ctx, inner: value }
        } else {
            panic!("Value is not js array");
        }
    }
}
impl_drop!(JsArray);
impl_clone!(JsArray);
impl_try_from!(JsValue for JsArray if v => v.is_array());

struct_type!(JsObject);
impl_type_common_fn!(
    JsObject,
    Option<JSValue>,
    crate::ffi::js_new_object_with_proto
);
impl<'a> std::fmt::Debug for JsObject<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("JsObject")
            .field(&self.raw_value().tag)
            .field(&"...")
            .finish()
    }
}
impl_drop!(JsObject);
impl_clone!(JsObject);
impl_try_from!(JsValue for JsObject if v => v.is_object());
impl<'a> JsObject<'a> {
    pub fn property(&self, name: &str) -> Option<JsValue<'a>> {
        // TODO: prevent allocation
        let cname = if let Ok(val) = crate::common::make_cstring(name) {
            val
        } else {
            return None;
        };

        let value = {
            let val = unsafe {
                crate::ffi::JS_GetPropertyStr(self.ctx.inner, self.inner, cname.as_ptr())
            };
            JsValue::new(self.ctx, val)
        };
        let tag = value.tag();

        if tag.is_exception() {
            None
        } else if tag.is_undefined() {
            None
        } else {
            Some(value)
        }
    }

    /// Determine if the object is a promise by checking the presence of
    /// a 'then' and a 'catch' property.
    pub fn is_promise(&self) -> bool {
        if let Some(p) = self.property("then") {
            if p.is_function() {
                return true;
            }
        }
        if let Some(p) = self.property("catch") {
            if p.is_function() {
                return false;
            }
        }
        false
    }

    pub fn set_property(&self, name: &str, value: JsValue<'a>) -> Result<(), crate::common::Error> {
        let cname = crate::common::make_cstring(name)?;
        unsafe {
            // NOTE: SetPropertyStr takes ownership of the value.
            // We do not, however, call JsValue::forget immediately, so
            // the inner JSValue is still managed.
            // `mem::forget` is called below only if SetProperty succeeds.
            // This prevents leaks when an error occurs.
            let ret = crate::ffi::JS_SetPropertyStr(
                self.ctx.inner,
                self.inner,
                cname.as_ptr(),
                value.inner,
            );

            if ret < 0 {
                Err(crate::common::Error::PropertyError(
                    "Could not set property".into(),
                ))
            } else {
                // Now we can call forget to prevent calling the destructor.
                std::mem::forget(value);
                Ok(())
            }
        }
    }
}

struct_type!(JsFunction);
impl<'a> JsFunction<'a> {
    pub fn call(&self, args: Vec<JsValue<'a>>) -> Result<JsValue<'a>, crate::common::Error> {
        let mut qargs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let len = qargs.len() as i32;

        let rst = unsafe {
            crate::ffi::JS_Call(
                self.ctx.inner,
                self.inner,
                crate::ffi::JS_UNDEFINED,
                len,
                qargs.as_mut_ptr(),
            )
        };

        let val = JsValue::new(self.ctx, rst);
        if val.is_exception() {
            if let Some(err) = get_last_exception(self.ctx) {
                Err(err)?
            } else {
                Err(Error::ExecuteError(
                    "JsFunction call() is failed".to_owned(),
                ))?
            }
        }

        Ok(val)
    }

    to_value_fn!();
}
impl_try_from!(JsValue for JsFunction if v => v.is_function());
impl_drop!(JsFunction);
impl_clone!(JsFunction);

struct_type!(JsCompiledFunction);
impl<'a> JsCompiledFunction<'a> {
    /// Evaluate this compiled function and return the resulting value.
    pub fn eval(&'a self) -> Result<JsValue<'a>, crate::common::Error> {
        let val = run_compiled_function(self)?;
        Ok(val)
    }

    /// Convert this compiled function into QuickJS bytecode.
    pub fn to_bytecode(&self) -> Result<Vec<u8>, Error> {
        Ok(to_bytecode(self.ctx, self))
    }

    to_value_fn!();
}
impl_try_from!(JsValue for JsCompiledFunction if v => v.is_compiled_function());
impl_drop!(JsCompiledFunction);
impl_clone!(JsCompiledFunction);

struct_type!(JsModule);
impl<'a> JsModule<'a> {
    to_value_fn!();
}
impl_try_from!(JsValue for JsModule if v => v.is_module());
impl_drop!(JsModule);
impl_clone!(JsModule);

// pub type NativeFunction<'a> = fn(
//     ctx: &'a crate::Context,
//     this_val: JsValue,
//     // argc: i32,
//     argv: &'a [JsValue],
// ) -> JsValue<'a>;

// pub type NativeFunctionMagic<'a> = fn(
//     ctx: &'a crate::Context,
//     this_val: JsValue,
//     // argc: i32,
//     argv: &'a [JsValue],
//     magic: i32,
// ) -> JsValue<'a>;

// pub type NativeFunctionData<'a> = fn(
//     ctx: &'a crate::Context,
//     this_val: JsValue,
//     // argc: i32,
//     argv: &'a [JsValue],
//     magic: i32,
//     func_data: &'a [JsValue],
// ) -> JsValue<'a>;

#[cfg(test)]
mod tests {
    use crate::{
        common::Error,
        function::{compile, js_eval, js_get_global_object},
        Context, Runtime,
    };

    use super::*;

    #[test]
    fn test_data() {
        let rt = Runtime::default();
        let ctx = &Context::new(rt);

        let val_1 = JsInteger::new(ctx, 2);
        let val_2 = JsInteger::new(ctx, 2);
        assert_eq!(val_1, val_2);

        let val_1 = JsInteger::new(ctx, 2);
        let val_2 = JsInteger::new(ctx, 3);
        assert_ne!(val_1, val_2);

        let val_1 = JsNumber::new(ctx, 3_f64);
        let val_2 = val_1.clone().to_value();
        let val_3: JsNumber = val_2.to_number().unwrap();
        assert_eq!(val_1, val_3);

        let val_1 = JsNumber::new(ctx, 3.14);
        let val_2 = val_1.clone().to_value();
        let val_3: JsNumber = val_2.to_number().unwrap();
        assert_eq!(val_1, val_3);

        let val_1 = JsNumber::new(ctx, 3.14);
        let val_2 = JsNumber::new(ctx, 3_f64);
        assert_ne!(val_1, val_2);

        let val_1 = JsString::new(&ctx, "abc");
        let val_2 = val_1.clone().to_value();
        let val_3: JsString = val_2.to_string().unwrap();
        assert_eq!(val_1, val_3);

        let val_1 = JsString::new(&ctx, "abc");
        let val_2 = JsString::new(&ctx, "abc");
        assert_eq!(val_1, val_2);

        let val_1 = JsString::new(&ctx, "ab1");
        let val_2 = JsString::new(&ctx, "ab2");
        assert_ne!(val_1, val_2);

        let js_val = unsafe { val_1.forget() };
        unsafe { crate::ffi::js_free_value(ctx.inner, js_val) };

        let js_prop = JsObject::new(ctx, None);
        let prop_val = JsInteger::new(ctx, 2);
        js_prop.set_property("name", prop_val.into());
        let val = js_prop.property("name").unwrap().to_int().unwrap();
        assert_eq!(2, val.value());
        let val = js_prop.property("name1");
        assert!(val.is_none());

        let script = "function add(a) { return a + 1; }";
        let js_val = js_eval(
            ctx,
            script,
            "<input>",
            crate::ffi::JS_EVAL_TYPE_GLOBAL as i32,
        )
        .unwrap();
        let global_obj = js_get_global_object(ctx).unwrap().to_object().unwrap();
        let js_fn = global_obj.property("add").unwrap();
        assert!(js_fn.is_function());

        let js_fn: JsFunction = js_fn.try_into().unwrap();
        let rst = js_fn
            .call(vec![JsInteger::new(ctx, 2).into()])
            .unwrap()
            .to_int()
            .unwrap()
            .value();
        assert_eq!(3, rst);

        let script = "{5 + 1;}";
        let js_compiled_fn: JsCompiledFunction =
            compile(ctx, script, "<input>").unwrap().try_into().unwrap();
        let js_compiled_val = js_compiled_fn.to_value();
        let js_compiled_fn: JsCompiledFunction = js_compiled_val.try_into().unwrap();
        let rst = js_compiled_fn.eval().unwrap().to_int().unwrap().value();
        assert_eq!(6, rst);
    }
}
