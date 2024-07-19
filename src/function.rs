use crate::{
    common::{make_cstring, Error}, Context, JsFunction, JsValue
};

pub fn js_eval<'a>(
    ctx: &'a Context,
    code: &str,
    file_name: &str,
    eval_flags: u32,
) -> Result<JsValue<'a>, Error> {
    let len = code.as_bytes().len();
    let code = make_cstring(code)?;
    let file_name = make_cstring(file_name)?;
    let val = unsafe {
        crate::ffi::JS_Eval(
            ctx.inner,
            code.as_ptr(),
            len,
            file_name.as_ptr(),
            eval_flags as i32,
        )
    };

    Ok(JsValue::new(ctx, val))
}

pub fn js_get_global_object<'a>(ctx: &'a Context) -> Result<JsValue<'a>, Error> {
    let val = unsafe { crate::ffi::JS_GetGlobalObject(ctx.inner) };
    let val = JsValue::new(ctx, val);
    if val.is_exception() {
        Err(Error::GeneralError("Can't get global object".to_owned()))?
    }
    
    Ok(val)
}