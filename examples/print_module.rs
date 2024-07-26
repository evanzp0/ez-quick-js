use std::fs;

use anyhow::Error;
use ez_quick_js::function::C_FUNC_DEF;
use ez_quick_js::JS_UNDEFINED;
use ez_quick_js::ffi::{JS_EVAL_TYPE_GLOBAL, JS_EVAL_TYPE_MODULE};
use ez_quick_js::{
    ffi::{JS_ToStr, JSCFunctionListEntry, JSContext, JSModuleDef, JSValue},
    function::{add_module_export_list, set_module_export_list},
    Context, JsModuleDef, Runtime,
};

fn main() -> Result<(), Error> {
    // load js script
    let file_name = "./examples/print_module.js";
    let code = &fs::read_to_string(file_name)?;
    // println!("{code}");

    let rt = Runtime::new(None);
    let ctx = &rt.create_context();


    // 初始化模块 m
    init_module(ctx, "m")?;

    println!("Eval script:");
    ctx.eval(code, file_name, (JS_EVAL_TYPE_GLOBAL | JS_EVAL_TYPE_MODULE) as i32)?;

    Ok(())
}

/// 创建模块并导出对象
fn init_module<'a>(ctx: &'a Context, module_name: &str) -> Result<JsModuleDef<'a>, Error> {
    // 创建模块，并初始化模块内本地对象
    let m = ctx.new_module(module_name, Some(init_module_inner_object))?;

    // 导出 tab (JS_FUNC_LIST) 列表中同名的本地对象
    add_module_export_list(ctx, &m, JS_FUNC_LIST.as_ref())?;

    Ok(m)
}

/// 初始化模块内的本地对象(函数、对象等)
unsafe extern "C" fn init_module_inner_object(
    ctx: *mut JSContext,
    m: *mut JSModuleDef,
) -> ::std::os::raw::c_int {
    // 生成模块内的本地对象
    set_module_export_list(ctx, m, JS_FUNC_LIST.as_ref())
}

const JS_FUNC_LIST: &[JSCFunctionListEntry] = &[
    C_FUNC_DEF(b"Print\0", 1, Some(js_print))
];

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
    ctx: *mut JSContext,
    _this_val: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    let args = std::slice::from_raw_parts(argv, argc as usize);

    for (idx, item) in args.iter().enumerate() {
        if idx != 0 {
            print!(" ");
        }

        let str = JS_ToStr(ctx, *item);
        print!("{str}");
    }

    println!();

    JS_UNDEFINED
}
