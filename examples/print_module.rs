use std::fs;

use anyhow::Error;
use ez_quick_js::c_func_def;
use ez_quick_js::ffi::{JS_EVAL_TYPE_GLOBAL, JS_EVAL_TYPE_MODULE};
use ez_quick_js::{
    ffi::{js_to_string, JSCFunctionListEntry, JSContext, JSModuleDef, JSValue, JS_UNDEFINED},
    function::{add_module_export_list, js_set_module_export_list},
    Context, JsModuleDef, Runtime,
};

fn main() -> Result<(), Error> {
    // load js script
    let file_name = "./examples/print_module.js";
    let code = &fs::read_to_string(file_name)?;
    // println!("{code}");

    let ctx = &Runtime::new(None).create_context();

    // 初始化模块 m
    init_module(ctx, "m")?;

    println!("Eval script:");
    ctx.eval(code, file_name, (JS_EVAL_TYPE_GLOBAL | JS_EVAL_TYPE_MODULE) as i32)?;

    Ok(())
}

const JS_FUNC_LIST: &[JSCFunctionListEntry] = &[c_func_def!("Print", 1, Some(js_print))];

/// 创建模块并导出对象
fn init_module(ctx: &Context, module_name: &str) -> Result<JsModuleDef, Error> {
    // 创建模块，并初始化模块内本地对象
    let m = ctx.new_module(module_name, Some(init_module_inner_object))?;

    // 导出 tab (JS_FUNC_LIST) 列表中同名的本地对象
    unsafe { add_module_export_list(ctx, &m, JS_FUNC_LIST)? };

    Ok(m)
}

/// 初始化模块内的本地对象(函数、对象等)
unsafe extern "C" fn init_module_inner_object(
    ctx: *mut JSContext,
    m: *mut JSModuleDef,
) -> ::std::os::raw::c_int {
    // 生成模块内的本地对象
    js_set_module_export_list(ctx, m, JS_FUNC_LIST)
}

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

        let str = js_to_string(ctx, *item);
        print!("{str}");
    }

    println!();

    JS_UNDEFINED
}
