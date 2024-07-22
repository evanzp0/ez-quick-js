use std::{f32::consts, ffi::c_void};

use crate::{
    common::{make_cstring, Error},
    ffi::{
        js_free, js_new_object_with_proto, JSCFunction, JSCFunctionEnum_JS_CFUNC_constructor, JSCFunctionEnum_JS_CFUNC_generic, JSCFunctionMagic, JSContext, JSValue, JS_Call, JS_DefinePropertyValue, JS_EvalFunction, JS_GetException, JS_NewAtomLen, JS_NewCFunction2, JS_ReadObject, JS_WriteObject, JS_READ_OBJ_BYTECODE, JS_UNDEFINED, JS_WRITE_OBJ_BYTECODE, NULL_PTR
    },
    Context, JsAtom, JsCompiledFunction, JsFunction, JsValue,
};

pub fn js_eval<'a>(
    ctx: &'a Context,
    code: &str,
    file_name: &str,
    eval_flags: i32,
) -> Result<JsValue<'a>, Error> {
    let code = make_cstring(code)?;
    let len = code.count_bytes();
    let file_name = make_cstring(file_name)?;

    let val = unsafe {
        crate::ffi::JS_Eval(
            ctx.inner,
            code.as_ptr(),
            len,
            file_name.as_ptr(),
            eval_flags,
        )
    };

    let val = JsValue::new(ctx, val);
    assert_exception(ctx, &val, "JS_Eval() is failed")?;

    Ok(val)
}

pub fn js_get_global_object<'a>(ctx: &'a Context) -> Result<JsValue<'a>, Error> {
    let val = unsafe { crate::ffi::JS_GetGlobalObject(ctx.inner) };
    let val = JsValue::new(ctx, val);
    assert_exception(ctx, &val, "Can't get global object")?;

    Ok(val)
}

/// Get the last exception from the runtime, and if present, convert it to a Error.
pub fn get_last_exception<'a>(ctx: &Context) -> Option<Error> {
    let value = unsafe {
        let raw = JS_GetException(ctx.inner);
        JsValue::new(ctx, raw)
    };

    if value.is_null() {
        None
    } else if value.is_exception() {
        Some(Error::GeneralError(
            "Could get exception from runtime".into(),
        ))
    } else {
        match value.to_string() {
            Ok(strval) => {
                let val = strval.value();
                if val.contains("out of memory") {
                    Some(Error::OutOfMemoryError)
                } else {
                    Some(Error::GeneralError(val.to_string()))
                }
            }
            Err(e) => Some(e),
        }
    }
}

/// compile a script, will result in a JSValueRef with tag JS_TAG_FUNCTION_BYTECODE or JS_TAG_MODULE.
///  It can be executed with run_compiled_function().
pub fn compile<'a>(ctx: &'a Context, script: &str, file_name: &str) -> Result<JsValue<'a>, Error> {
    js_eval(
        ctx,
        script,
        file_name,
        crate::ffi::JS_EVAL_FLAG_COMPILE_ONLY as i32,
    )
}

/// run a compiled function, see compile for an example
pub fn run_compiled_function<'a>(func: &'a JsCompiledFunction) -> Result<JsValue<'a>, Error> {
    let ctx = func.ctx;
    let val = unsafe {
        // NOTE: JS_EvalFunction takes ownership.
        // We clone the func and extract the inner JsValue by forget().
        let f = func.clone().to_value().forget();
        let v = JS_EvalFunction(ctx.inner, f);

        v
    };

    let val = JsValue::new(ctx, val);
    assert_exception(ctx, &val, "Could not evaluate compiled function")?;

    Ok(val)
}

/// write a function to bytecode
pub fn to_bytecode<'a>(ctx: &'a Context, compiled_func: &JsCompiledFunction) -> Vec<u8> {
    unsafe {
        let mut len = 0;
        let raw = JS_WriteObject(
            ctx.inner,
            &mut len,
            compiled_func.inner,
            JS_WRITE_OBJ_BYTECODE as i32,
        );
        let slice = std::slice::from_raw_parts(raw, len as usize);
        let data = slice.to_vec();
        js_free(ctx.inner, raw as *mut c_void);

        data
    }
}

/// read a compiled function from bytecode, see to_bytecode for an example
pub fn from_bytecode<'a>(ctx: &'a Context, bytecode: &[u8]) -> Result<JsValue<'a>, Error> {
    if bytecode.is_empty() {
        Err(Error::GeneralError(
            "from_bytecode() failed, bytecode length is 0".to_owned(),
        ))?
    }

    let len = bytecode.len();
    let buf = bytecode.as_ptr();
    let raw = unsafe { JS_ReadObject(ctx.inner, buf, len as _, JS_READ_OBJ_BYTECODE as i32) };

    let func = JsValue::new(ctx, raw);
    assert_exception(
        ctx,
        &func,
        "from_bytecode() failed and could not get exception",
    )?;

    Ok(func)
}

pub fn new_cfunction<'a>(
    ctx: &'a Context,
    func: JSCFunction,
    name: &str,
    arg_count: i32,
) -> Result<JsValue<'a>, Error> {
    new_cfunction2(ctx, func, name, arg_count, false)
}

pub fn new_cfunction2<'a>(
    ctx: &'a Context,
    func: JSCFunction,
    name: &str,
    arg_count: i32,
    is_constructor: bool,
) -> Result<JsValue<'a>, Error> {
    new_cfunction_magic(ctx, func, name, arg_count, false, 0)
}

pub fn new_cfunction_magic<'a>(
    ctx: &'a Context,
    func: JSCFunction,
    name: &str,
    arg_count: i32,
    is_constructor: bool,
    magic: i32,
) -> Result<JsValue<'a>, Error> {
    let name = make_cstring(name)?;

    let cproto = if is_constructor {
        JSCFunctionEnum_JS_CFUNC_constructor
    } else {
        JSCFunctionEnum_JS_CFUNC_generic
    };

    let value = unsafe {
        crate::ffi::JS_NewCFunction2(ctx.inner, func, name.as_ptr(), arg_count, cproto, magic)
    };

    Ok(JsValue::new(ctx, value))
}

pub fn get_global_object<'a>(ctx: &'a Context) -> JsValue<'a> {
    let val = unsafe { crate::ffi::JS_GetGlobalObject(ctx.inner) };
    JsValue::new(ctx, val)
}

pub fn new_object_with_proto<'a>(
    ctx: &'a Context,
    proto: Option<JsValue>,
) -> Result<JsValue<'a>, Error> {
    let proto = proto.map(|val| val.inner);
    let val = unsafe { js_new_object_with_proto(ctx.inner, proto) };

    let val = JsValue::new(ctx, val);
    assert_exception(
        ctx,
        &val,
        "new_object_with_proto() failed and could not get exception",
    )?;

    Ok(val)
}

pub fn new_atom<'a>(ctx: &'a Context, name: &str) -> Result<JsAtom<'a>, Error> {
    let name = make_cstring(name)?;

    let atom = unsafe { JS_NewAtomLen(ctx.inner, name.as_ptr(), name.count_bytes()) };
    let atom = JsAtom::new(ctx, atom);
    if atom.is_exception() {
        if let Some(err) = get_last_exception(ctx) {
            Err(err)?
        } else {
            Err(Error::GeneralError("New atom failed".to_string()))?
        }
    }

    Ok(atom)
}

pub fn define_property(
    ctx: &Context,
    this_obj: &JsValue,
    prop_name: JsAtom,
    prop_value: JsValue,
    flags: i32,
) -> Result<(), Error> {
    let val = unsafe {
        JS_DefinePropertyValue(
            ctx.inner,
            this_obj.inner,
            prop_name.inner,
            prop_value.inner,
            flags,
        )
    };

    if val == -1 {
        if let Some(err) = get_last_exception(ctx) {
            Err(err)?
        } else {
            Err(Error::GeneralError(
                "define_property() is failed".to_string(),
            ))?
        }
    }

    Ok(())
}

pub fn assert_exception(ctx: &Context, val: &JsValue, err_msg: &str) -> Result<(), Error> {
    Ok(if val.is_exception() {
        let rst = get_last_exception(ctx);
        if let Some(err) = rst {
            Err(err)?
        } else {
            Err(Error::GeneralError(err_msg.to_string()))?
        }
    })
}

pub fn call_function<'a>(
    ctx: &'a Context,
    func: &JsValue,
    this_obj: Option<&JsValue>,
    argv: &[&JsValue],
) -> Result<JsValue<'a>, Error> {
    let argc = argv.len() as i32;
    let this_val = if let Some(val) = this_obj {
        val.inner
    } else {
        JS_UNDEFINED
    };
    
    let mut qargs = argv
        .iter()
        .map(|a| a.inner)
        .collect::<Vec<_>>();
    // let qargs = {
    //     let ptr = &mut ctx.get_int(12).inner;
    //     let ptr = ptr as *mut JSValue;
    //     ptr
    // };
    
    let val = unsafe { JS_Call(ctx.inner, func.inner, this_val, argc, qargs.as_mut_ptr()) };
    let val = JsValue::new(ctx, val);
    assert_exception(ctx, &val, "call_function() is failed")?;

    Ok(val)
}

#[cfg(test)]
mod tests {
    use crate::{JsInteger, Runtime};

    use super::*;

    #[test]
    fn test_compile_and_run() {
        let rt = Runtime::default();
        let ctx = &Context::new(rt);

        let script = "{let a = 7; let b = 5; a * b;}";
        let js_compiled_val: JsCompiledFunction =
            compile(ctx, script, "<test>").unwrap().try_into().unwrap();
        let bytes = to_bytecode(ctx, &js_compiled_val);
        let js_compiled_val: JsCompiledFunction =
            from_bytecode(ctx, &bytes).unwrap().try_into().unwrap();
        println!("{:?}", js_compiled_val.clone().to_value().tag());

        let rst: JsInteger = run_compiled_function(&js_compiled_val)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(rst.value(), 7 * 5);
    }
}
