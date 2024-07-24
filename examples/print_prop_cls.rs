use std::cell::Cell;
use std::ffi::CStr;
use std::fs;
use std::ptr::null_mut;

use anyhow::Error;
use ez_quick_js::ffi::{
    js_to_string, JSClassDef, JSClassID, JSRuntime, JS_GetOpaque, JS_GetOpaque2, JS_ToInt32, JS_EVAL_TYPE_GLOBAL, JS_TAG_INT
};
use ez_quick_js::function::{
    new_c_function2, new_class, new_class_id, new_object_proto_class, set_class_proto,
    set_constructor, set_property_function_list, C_FUNC_DEF, C_GET_SET_DEF,
};
use ez_quick_js::{
    ffi::{JSCFunctionListEntry, JSContext, JSValue},
    Context, Runtime,
};
use ez_quick_js::{JsValue, JS_EXCEPTION, JS_NULL, JS_UNDEFINED};

#[derive(Debug, Clone)]
struct PrintClass {
    val: i32,
}

impl Drop for PrintClass {
    fn drop(&mut self) {
        println!("PrintClass is drop");
    }
}

impl PrintClass {
    pub fn set_val(&mut self, val: i32) {
        self.val = val;
    }
}

static mut PRINT_CLASS_ID: JSClassID = 0;

unsafe extern "C" fn js_print_cls_finalizer(_rt: *mut JSRuntime, val: JSValue) {
    let native_print = JS_GetOpaque(val, PRINT_CLASS_ID) as *mut Cell<PrintClass>;

    println!(
        "js_print_cls_finalizer run: {}",
        (*native_print).get_mut().val
    );

    if native_print != std::ptr::null_mut() {
        let _ = Box::from_raw(native_print);
        // js_free_rt(rt, native_print);
    }
}

const PRINT_CLASS_DEF: JSClassDef = JSClassDef {
    class_name: b"Print\0".as_ptr() as _,
    finalizer: Some(js_print_cls_finalizer), // 析构函数
    gc_mark: None,
    call: None,
    exotic: std::ptr::null_mut(),
};

#[allow(unused_assignments)]
/// PrintClass constructor 的主逻辑，返回值为一个 Print （JSValue）实例对象，
/// 并将该对象和 native 对象关联
fn js_printclass_constructor2_inner<'a>(
    ctx: &'a Context,
    new_target: &JsValue,
    args: &[JsValue],
) -> Result<JSValue, ez_quick_js::common::Error> {
    println!("PrintClass constructor is called");

    // let is_exception = Cell::new(false);
    //
    // 生成 native 对象
    // let native_print = {
    //     // js_malloc() 是在 JS 内部堆上分配内存，受到 JS Runtime 堆容量的限制(默认是无限大小)
    //     let val = unsafe { js_malloc(ctx.inner, size_of::<PrintClass>()) } as *const PrintClass;
    //     if val == null_mut() {
    //         Err(ez_quick_js::common::Error::GeneralError(
    //             "js_malloc() is failed when alloc print class".to_owned(),
    //         ))?
    //     }
    // };
    //
    // 发生异常时，释放 native_print
    // scopeguard::defer!(unsafe {
    //     if is_exception.get() {
    //         js_free(ctx.inner, native_print as _);
    //         println!("js_free native_print");
    //     }
    // });

    let mut native_print = Box::new(Cell::new(PrintClass { val: 0 }));

    if args.len() > 0 {
        let val = args[0].clone().to_int()?.value();
        // unsafe { (*(native_print as *mut PrintClass)).val = val; }
        native_print.get_mut().val = val;
    }

    // 获取 new_target 的 prototype
    let proto = new_target
        .get_property("prototype")
        .ok_or(ez_quick_js::common::Error::GeneralError(
            "JS_GetPropertyStr() is failed when get 'prototype'".to_owned(),
        ))
        .map_err(|err| {
            // is_exception.set(true);
            err
        })?;

    let cls_id = unsafe { PRINT_CLASS_ID };
    let js_print_obj =
        new_object_proto_class(ctx, &proto /*也可以设为 JS_NULL*/, cls_id).map_err(|err| {
            // is_exception.set(true);
            err
        })?;

    // 没有发生异常，则不释放 native_print
    js_print_obj.set_opaque(Box::into_raw(native_print) as _);
    // js_print_obj.set_opaque(native_print as _);

    Ok(unsafe { js_print_obj.forget() })
}

/// PrintClass Constructor 的包装函数
unsafe extern "C" fn js_printclass_constructor2(
    ctx: *mut JSContext,
    new_target: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    let ctx = Context::from_raw(ctx);
    let new_target = JsValue::new(&ctx, new_target);

    let rst = {
        let args = {
            let tmp = std::slice::from_raw_parts(argv, argc as usize);
            let tmp = tmp
                .iter()
                .map(|val| JsValue::new(&ctx, *val))
                .collect::<Vec<JsValue>>();
            tmp
        };

        js_printclass_constructor2_inner(&ctx, &new_target, &args)
    };

    let ret_val = if let Ok(val) = rst { val } else { JS_EXCEPTION };

    new_target.forget();
    ctx.forget();

    ret_val
}

unsafe extern "C" fn js_print_test_func(
    ctx: *mut JSContext,
    this_val: JSValue,
    _argc: ::std::os::raw::c_int,
    _argv: *mut JSValue,
) -> JSValue {
    let native_print = JS_GetOpaque2(ctx, this_val, PRINT_CLASS_ID) as *mut Cell<PrintClass>;

    if native_print == null_mut() {
        return JS_EXCEPTION;
    }
    println!("Print Value ~: {}", (*native_print).get_mut().val);

    JS_UNDEFINED
}

unsafe extern "C" fn js_print_val_getter(ctx: *mut JSContext, this_val: JSValue) -> JSValue {
    let ctx = Context::from_raw(ctx);

    let native_print = JS_GetOpaque2(ctx.inner, this_val, PRINT_CLASS_ID) as *mut Cell<PrintClass>;

    if native_print == null_mut() {
        ctx.forget();
        return JS_EXCEPTION;
    }

    let val = ctx.get_int((*native_print).get_mut().val).forget();
    ctx.forget();

    println!("Print val getter is called");

    return val;
}

unsafe extern "C" fn js_print_val_setter(
    ctx: *mut JSContext,
    this_val: JSValue,
    val: JSValue,
) -> JSValue {
    let native_print: *mut PrintClass = JS_GetOpaque2(ctx, this_val, PRINT_CLASS_ID) as _;
    if native_print == null_mut() {
        return JS_EXCEPTION;
    }

    let mut param = 0;
    if val.tag != JS_TAG_INT.into() {
        return JS_EXCEPTION;
    } else {
        if JS_ToInt32(ctx, &mut param, val) != 0 {
            return JS_EXCEPTION;
        } 
    }

    // 调用 native 方法
    print_val_setter(native_print.as_mut().unwrap(),  param);

    println!("Print val setter is called");

    return JS_UNDEFINED;
}

fn print_val_setter(print: &mut PrintClass, val: i32) {
    print.set_val(val);
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
        Some(js_printclass_constructor2),
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

///// Print 构造函数, 返回值为一个 Print （JsValue）实例对象，并将该和 native 对象关联
// unsafe extern "C" fn js_printclass_constructor(
//     ctx: *mut JSContext,
//     new_target: JSValue,
//     argc: ::std::os::raw::c_int,
//     argv: *mut JSValue,
// ) -> JSValue {
//     println!("PrintClass constructor is called");

//     let args = std::slice::from_raw_parts(argv, argc as usize);

//     // 生成 native 对象
//     let native_print = js_malloc(ctx, size_of::<PrintClass>()) as *mut PrintClass;
//     if native_print == null_mut() {
//         return JS_EXCEPTION;
//     }

//     if argc > 0 {
//         let mut tmp = 0;
//         let rst = JS_ToInt32(ctx, &mut tmp as _, args[0]);
//         (*native_print).val = tmp;

//         if rst != 0 {
//             js_free(ctx, native_print as _);
//             return JS_EXCEPTION;
//         }
//     }

//     // 获取 new_target 的 prototype
//     let proto = JS_GetPropertyStr(ctx, new_target, b"prototype\0".as_ptr() as _);
//     if js_is_exception(proto) {
//         js_free(ctx, native_print as _);
//         return JS_EXCEPTION;
//     }

//     // 用 proto 对应的 shape 生成一个新 JS 实例对象，该对象的 class_id 为 print_class_id, prototype 为 proto
//     let js_print_obj = JS_NewObjectProtoClass(
//         ctx,
//         proto, /* 也可以设为 JS_NULL */
//         PRINT_CLASS_ID,
//     );
//     JS_SetOpaque(js_print_obj, native_print as _);

//     js_free_value(ctx, proto);

//     js_print_obj
// }
// 
// unsafe extern "C" fn js_print_val_setter(
//     ctx: *mut JSContext,
//     this_val: JSValue,
//     val: JSValue,
// ) -> JSValue {
//     let ctx = Context::from_raw(ctx);

//     let native_print: *mut PrintClass = JS_GetOpaque2(ctx.inner, this_val, PRINT_CLASS_ID) as _;
//     if native_print == null_mut() {
//         ctx.forget();
//         return JS_EXCEPTION;
//     }

//     if val.tag != JS_TAG_INT.into() {
//         ctx.forget();
//         return JS_EXCEPTION;
//     } else {
//         let val = JsValue::new(&ctx, val);
//         let val: JsInteger = val.try_into().unwrap();
//         (*native_print).val = val.value();
//     }

//     ctx.forget();

//     println!("Print val setter is called");

//     return JS_UNDEFINED;
// }