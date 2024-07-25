use std::fs;

use anyhow::Error;
use ez_quick_js::ffi::{
    JS_EVAL_TYPE_GLOBAL, JS_PROP_CONFIGURABLE, JS_PROP_C_W_E, JS_PROP_WRITABLE,
};
use ez_quick_js::function::{
    define_property_str, set_property_function_list, C_FUNC_DEF, OBJECT_DEF,
};
use ez_quick_js::{
    ffi::{JSCFunctionListEntry, JSContext, JSValue},
    Context, Runtime,
};
use ez_quick_js::{JsValue, JS_UNDEFINED};

fn main() -> Result<(), Error> {
    // load js script
    let file_name = "./examples/print_property.js";
    let code = &fs::read_to_string(file_name)?;
    // println!("{code}");

    let rt = Runtime::new(None);
    let ctx = &rt.create_context();


    let global_obj = ctx.get_global_object();
    init_register_property(ctx, &global_obj)?;

    println!("Eval script:");
    let _rst = ctx.eval(
        code,
        file_name,
        (JS_EVAL_TYPE_GLOBAL) as i32,
    )?;

    // println!("_rst = {:?}", _rst.to_int().unwrap().value());

    Ok(())
}

fn init_register_property(ctx: &Context, global_obj: &JsValue) -> Result<(), Error> {
    let obj = ctx.new_object()?;
    set_property_function_list(ctx, &obj, JS_PROP_LIST.as_ref());

    define_property_str(ctx, global_obj, "reg", obj, JS_PROP_C_W_E as i32)?;

    let prop_print = ctx.get_cfunction(js_print, "Print", 1)?;
    global_obj.set_property("Print", prop_print)?;

    Ok(())
}

const JS_PROP_LIST: &[JSCFunctionListEntry] = &[OBJECT_DEF(
    b"m\0",
    JS_FUNC_LIST,
    JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
)];

// const JS_PROP_LIST: [JSCFunctionListEntry; 1] = [JSCFunctionListEntry {
//     name: b"m\0".as_ptr() as _,
//     prop_flags: (JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE) as u8,
//     def_type: ez_quick_js::ffi::JS_DEF_OBJECT as u8,
//     magic: 0,
//     u: ez_quick_js::ffi::JSCFunctionListEntry__bindgen_ty_1 {
//         prop_list: ez_quick_js::ffi::JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_4 {
//             tab: JS_FUNC_LIST.as_ptr(),
//             len: 1,
//         },
//     },
// }];

const JS_FUNC_LIST: &[JSCFunctionListEntry] = &[C_FUNC_DEF(b"Print\0", 1, Some(js_print))];

// const JS_FUNC_LIST: [JSCFunctionListEntry; 1] = [
//     JSCFunctionListEntry {
//         name: b"Print\0".as_ptr() as _,
//         prop_flags: (ez_quick_js::ffi::JS_PROP_WRITABLE | ez_quick_js::ffi::JS_PROP_CONFIGURABLE) as u8,
//         def_type: ez_quick_js::ffi::JS_DEF_CFUNC as u8,
//         magic: 0,
//         u: ez_quick_js::ffi::JSCFunctionListEntry__bindgen_ty_1 {
//             func: ez_quick_js::ffi::JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_1 {
//                 length: 1,
//                 cproto: ez_quick_js::ffi::JSCFunctionEnum_JS_CFUNC_generic as u8,
//                 cfunc: ez_quick_js::ffi::JSCFunctionType { generic: Some(js_print) },
//             },
//         },
//     }
// ];

unsafe extern "C" fn js_print(
    _ctx: *mut JSContext,
    _this_val: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    let args = std::slice::from_raw_parts(argv, argc as usize);

    for (idx, item) in args.iter().enumerate() {
        if idx != 0 {
            print!(" ");
        }
        let str = ez_quick_js::ffi::JS_ToStr(_ctx, *item);
        print!(" {str} ");
    }

    println!();

    JS_UNDEFINED
}
