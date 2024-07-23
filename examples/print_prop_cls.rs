use std::ffi::CStr;
use std::fs;
use std::mem::size_of;
use std::ptr::null_mut;

use anyhow::Error;
use ez_quick_js::ffi::{
    js_free, js_free_rt, js_free_value, js_is_exception, js_malloc, js_to_string, JSClassDef,
    JSClassID, JSRuntime, JS_GetOpaque, JS_GetOpaque2, JS_GetPropertyStr, JS_NewObjectProtoClass,
    JS_SetOpaque, JS_ToInt32, JS_EVAL_TYPE_GLOBAL, JS_TAG_INT,
};
use ez_quick_js::function::{
    new_c_function2, new_class, new_class_id, set_class_proto,
    set_constructor, set_property_function_list, C_FUNC_DEF, C_GET_SET_DEF,
};
use ez_quick_js::{
    ffi::{JSCFunctionListEntry, JSContext, JSValue},
    Context, Runtime,
};
use ez_quick_js::{JsInteger, JsValue, JS_EXCEPTION, JS_NULL, JS_UNDEFINED};

#[derive(Debug, Clone)]
struct PrintClass {
    val: i32,
}

static mut PRINT_CLASS_ID: JSClassID = 0;

unsafe extern "C" fn js_print_cls_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let native_print = JS_GetOpaque(val, PRINT_CLASS_ID) as _;

    if native_print == std::ptr::null_mut() {
        js_free_rt(rt, native_print);
    }
}

const PRINT_CLASS_DEF: JSClassDef = JSClassDef {
    class_name: b"Print\0".as_ptr() as _,
    finalizer: Some(js_print_cls_finalizer), // 析构函数
    gc_mark: None,
    call: None,
    exotic: std::ptr::null_mut(),
};

/// Print 构造函数, 返回值为一个 Print （JsValue）实例对象，并将该和 native 对象关联
unsafe extern "C" fn js_printclass_constructor(
    ctx: *mut JSContext,
    new_target: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    println!("PrintClass constructor is called");

    let args = std::slice::from_raw_parts(argv, argc as usize);

    // 生成 native 对象
    let native_print = js_malloc(ctx, size_of::<PrintClass>()) as *mut PrintClass;
    if native_print == null_mut() {
        return JS_EXCEPTION;
    }

    if argc > 0 {
        let mut tmp = 0;
        let rst = JS_ToInt32(ctx, &mut tmp as _, args[0]);
        (*native_print).val = tmp;

        if rst != 0 {
            js_free(ctx, native_print as _);
            return JS_EXCEPTION;
        }
    }

    // 获取 new_target 的 prototype
    let proto = JS_GetPropertyStr(ctx, new_target, b"prototype\0".as_ptr() as _);
    if js_is_exception(proto) {
        js_free(ctx, native_print as _);
        return JS_EXCEPTION;
    }

    // 用 proto 对应的 shape 生成一个新 JS 实例对象，该对象的 class_id 为 print_class_id, prototype 为 proto
    let js_print_obj = JS_NewObjectProtoClass(
        ctx,
        proto, /* 也可以设为 JS_NULL */
        PRINT_CLASS_ID,
    );
    JS_SetOpaque(js_print_obj, native_print as _);

    js_free_value(ctx, proto);

    js_print_obj
}

unsafe extern "C" fn js_print_test_func(
    ctx: *mut JSContext,
    this_val: JSValue,
    _argc: ::std::os::raw::c_int,
    _argv: *mut JSValue,
) -> JSValue {
    let native_print: *mut PrintClass = JS_GetOpaque2(ctx, this_val, PRINT_CLASS_ID) as _;
    if native_print == null_mut() {
        return JS_EXCEPTION;
    }

    println!("Print Value: {}", (*native_print).val);

    JS_UNDEFINED
}

unsafe extern "C" fn js_print_val_getter(ctx: *mut JSContext, this_val: JSValue) -> JSValue {
    let ctx = Context::from_raw(ctx);

    let native_print: *mut PrintClass = JS_GetOpaque2(ctx.inner, this_val, PRINT_CLASS_ID) as _;
    if native_print == null_mut() {
        ctx.forget();
        return JS_EXCEPTION;
    }

    let val = ctx.get_int((*native_print).val).forget();
    ctx.forget();

    println!("Print val getter is called");

    return val;
}

unsafe extern "C" fn js_print_val_setter(
    ctx: *mut JSContext,
    this_val: JSValue,
    val: JSValue,
) -> JSValue {
    let ctx = Context::from_raw(ctx);

    let native_print: *mut PrintClass = JS_GetOpaque2(ctx.inner, this_val, PRINT_CLASS_ID) as _;
    if native_print == null_mut() {
        ctx.forget();
        return JS_EXCEPTION;
    }

    if val.tag != JS_TAG_INT.into() {
        ctx.forget();
        return JS_EXCEPTION;
    } else {
        let val = JsValue::new(&ctx, val);
        let val: JsInteger = val.try_into().unwrap();
        (*native_print).val = val.value();
    }

    ctx.forget();

    println!("Print val setter is called");

    return JS_UNDEFINED;
}

const JS_PRINT_FUNCS: &[JSCFunctionListEntry] = &[
    C_FUNC_DEF(b"PrintTestFunc\0", 1, Some(js_print_test_func)),
    C_GET_SET_DEF(
        b"val\0",
        Some(js_print_val_getter),
        Some(js_print_val_setter),
    ),
];

#[allow(static_mut_refs)]
/// 在全局对象上注册 "Print" 类构造器
fn init_register_class(ctx: &Context, global_obj: &JsValue) -> Result<(), Error> {
    let class_id = unsafe { new_class_id(&mut PRINT_CLASS_ID) };
    new_class(ctx, class_id, &PRINT_CLASS_DEF)?;

    // let proto = ctx.new_prototype(JsValue::new(ctx, JS_NULL))?;
    let proto = unsafe { ez_quick_js::ffi::JS_NewObjectProto(ctx.inner, JS_NULL) };
    let proto = JsValue::new(ctx, proto);
    set_property_function_list(ctx, &proto, JS_PRINT_FUNCS.as_ref());

    let class_name = unsafe { CStr::from_ptr(PRINT_CLASS_DEF.class_name) };
    // JSValue ctor = JS_NewCFunction2(ctx, Js_PrintClass_Constructor, print_class_def.class_name, 1, JS_CFUNC_constructor, 0);
    let print_ctor = new_c_function2(
        ctx,
        Some(js_printclass_constructor),
        &class_name.to_string_lossy(),
        1,
        true,
    )?;
    set_constructor(ctx, &print_ctor, &proto)?;
    set_class_proto(ctx, class_id, &proto)?;

    // define_property_str(ctx, global_obj, "Print", print_ctor, JS_PROP_C_W_E as i32)?;
    global_obj.set_property("Print", print_ctor)?;

    unsafe {
        proto.forget();
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    // load js script
    let file_name = "./examples/print_prop_cls.js";
    let code = &fs::read_to_string(file_name)?;
    // println!("{code}");

    let ctx = &Runtime::new(None).create_context();
    let global_obj = ctx.get_global_object();

    init_register_class(ctx, &global_obj)?;
    add_global_print(ctx);

    println!("Eval script:");
    let _rst = ctx.eval(code, file_name, (JS_EVAL_TYPE_GLOBAL) as i32)?;

    Ok(())
}

fn add_global_print(ctx: &Context) {
    let global_obj = ctx.get_global_object();
    let console = ctx.new_object().unwrap();

    console
        .set_property("log", ctx.get_cfunction(js_print, "log", 1).unwrap())
        .unwrap();
    global_obj.set_property("console", console).unwrap();
    global_obj
        .set_property("print", ctx.get_cfunction(js_print, "log", 1).unwrap())
        .unwrap();
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
