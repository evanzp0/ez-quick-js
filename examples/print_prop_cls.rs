use std::cell::Cell;
use std::ffi::CStr;
use std::fs;
use std::ptr::null_mut;

use anyhow::Error;
use ez_quick_js::ffi::{
    js_free_value, js_is_exception, js_new_int32, js_to_string, JSClassDef, JSClassID, JSRuntime,
    JS_GetOpaque, JS_GetOpaque2, JS_GetPropertyStr, JS_NewObjectProtoClass, JS_SetOpaque,
    JS_ToInt32, JS_EVAL_TYPE_GLOBAL, JS_TAG_INT,
};
use ez_quick_js::function::{
    new_c_function2, new_class, new_class_id, set_class_proto, set_constructor,
    set_property_function_list, C_FUNC_DEF, C_GET_SET_DEF,
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
    pub fn new(val: i32) -> Self {
        Self { val }
    }

    pub fn set_val(&mut self, val: i32) {
        self.val = val;
    }

    pub fn get_val(&self) -> i32 {
        self.val
    }

    pub fn test_func(&self) {
        println!("Print Value ~: {}", self.val);
    }
}

/// JS 类相关 (class_id, 析构函数, class定义)
const PRINT_CLASS_ID: Cell<JSClassID> = Cell::new(0);

unsafe extern "C" fn js_print_cls_finalizer(_rt: *mut JSRuntime, val: JSValue) {
    let native_print = JS_GetOpaque(val, PRINT_CLASS_ID.get()) as *mut PrintClass;

    println!(
        "js_print_cls_finalizer run: {}",
        (*native_print).val
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

/// Print 构造函数, 返回值为一个 Print （JsValue）实例对象，并将该和 native 对象关联
/// new_target 是一个 ctor 对象(JS Function)，
///     它就是我们后面将 js_printclass_constructor 注册到全局对象上的那个 JS Function
unsafe extern "C" fn js_printclass_constructor(
    ctx: *mut JSContext,
    new_target: JSValue, // js ctor
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    println!("PrintClass constructor is called");

    // 提取参数并转换为 native 类型
    let args = std::slice::from_raw_parts(argv, argc as usize);
    let param = if argc > 0 {
        let mut tmp = 0;
        let rst = JS_ToInt32(ctx, &mut tmp as _, args[0]);
        if rst != 0 {
            return JS_EXCEPTION;
        }

        tmp
    } else {
        return JS_EXCEPTION;
    };

    // 生成 native 对象
    let native_print = {
        let val = native_print_constructor(param);
        Box::new(Cell::new(val))
    };

    let js_print_obj = {
        // 从 js ctor 上获取 prototype
        let proto = JS_GetPropertyStr(ctx, new_target, b"prototype\0".as_ptr() as _);
        if js_is_exception(proto) {
            return JS_EXCEPTION;
        }

        // 用 ctor.proto 对应的 shape 生成一个新 JS 实例对象，该对象的 class_id 为 print_class_id, prototype 为 proto
        let obj = JS_NewObjectProtoClass(
            ctx,
            proto, /* 也可以设为 JS_NULL */
            PRINT_CLASS_ID.get(),
        );
        js_free_value(ctx, proto);

        obj
    };

    // 用 Box::into_raw() leak native_print 对象，避免它被 drop
    JS_SetOpaque(js_print_obj, Box::into_raw(native_print) as _);

    js_print_obj
}
/// native constructor
fn native_print_constructor(val: i32) -> PrintClass {
    PrintClass::new(val)
}

unsafe extern "C" fn js_print_test_func(
    ctx: *mut JSContext,
    this_val: JSValue,
    _argc: ::std::os::raw::c_int,
    _argv: *mut JSValue,
) -> JSValue {
    // 获取 native 对象
    let native_print = {
        let tmp = JS_GetOpaque2(ctx, this_val, PRINT_CLASS_ID.get()) as *mut PrintClass;
        if tmp == null_mut() {
            return JS_EXCEPTION;
        }
        tmp
    };

    native_print_test_func(native_print.as_ref().unwrap());

    JS_UNDEFINED
}
/// native test_func
fn native_print_test_func(print: &PrintClass) {
    print.test_func();
}

unsafe extern "C" fn js_print_val_getter(ctx: *mut JSContext, this_val: JSValue) -> JSValue {
    // 获取 native 对象
    let native_print = {
        let tmp = JS_GetOpaque2(ctx, this_val, PRINT_CLASS_ID.get()) as *mut PrintClass;
        if tmp == null_mut() {
            return JS_EXCEPTION;
        }
        tmp
    };

    // 获取 print.val 并转成 JSValue 类型
    let val = native_print_val_getter(native_print.as_ref().unwrap());
    let js_val = js_new_int32(ctx, val);

    println!("Print val getter is called");

    js_val
}
/// native getter
fn native_print_val_getter(print: &PrintClass) -> i32 {
    print.get_val()
}

unsafe extern "C" fn js_print_val_setter(
    ctx: *mut JSContext,
    this_val: JSValue,
    val: JSValue,
) -> JSValue {
    let native_print: *mut PrintClass = JS_GetOpaque2(ctx, this_val, PRINT_CLASS_ID.get()) as _;
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
    native_print_val_setter(native_print.as_mut().unwrap(), param);

    println!("Print val setter is called");

    return JS_UNDEFINED;
}
/// native setter
fn native_print_val_setter(print: &mut PrintClass, val: i32) {
    print.set_val(val);
}

/// JS 函数列表, 用来添加到 JS Ojbect 上
const JS_PRINT_FUNCS: &[JSCFunctionListEntry] = &[
    C_FUNC_DEF(b"PrintTestFunc\0", 1, Some(js_print_test_func)),
    C_GET_SET_DEF(
        b"val\0",
        Some(js_print_val_getter),
        Some(js_print_val_setter),
    ),
];

#[allow(const_item_mutation)]
/// 在全局对象上注册 "Print" 类构造器
fn init_register_class(ctx: &Context, global_obj: &JsValue) -> Result<(), Error> {
    let class_id = new_class_id(PRINT_CLASS_ID.get_mut());
    new_class(ctx, class_id, &PRINT_CLASS_DEF)?;

    // 生成一个 JS Prototype 对象
    let proto = ctx.new_prototype(JsValue::new(ctx, JS_NULL))?;
    // 将成员方法都关联到 proto 上
    set_property_function_list(ctx, &proto, JS_PRINT_FUNCS.as_ref());

    let class_name = unsafe { CStr::from_ptr(PRINT_CLASS_DEF.class_name) };
    let print_ctor = new_c_function2(
        ctx,
        Some(js_printclass_constructor),
        &class_name.to_string_lossy(),
        1,
        true,
    )?;

    // set ctor.prototype and prototype.constructor
    // 设置原型链，在 context 的 class 表的 class_id 项上注册 proto
    {
        set_constructor(ctx, &print_ctor, &proto)?;
        set_class_proto(ctx, class_id, &proto)?;
    }

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

// #[allow(unused_assignments)]
// /// PrintClass constructor 的主逻辑，返回值为一个 Print （JSValue）实例对象，
// /// 并将该对象和 native 对象关联
// fn js_printclass_constructor2_inner<'a>(
//     ctx: &'a Context,
//     new_target: &JsValue,
//     args: &[JsValue],
// ) -> Result<JSValue, ez_quick_js::common::Error> {
//     println!("PrintClass constructor is called");

//     // let is_exception = Cell::new(false);
//     //
//     // 生成 native 对象
//     // let native_print = {
//     //     // js_malloc() 是在 JS 内部堆上分配内存，受到 JS Runtime 堆容量的限制(默认是无限大小)
//     //     let val = unsafe { js_malloc(ctx.inner, size_of::<PrintClass>()) } as *const PrintClass;
//     //     if val == null_mut() {
//     //         Err(ez_quick_js::common::Error::GeneralError(
//     //             "js_malloc() is failed when alloc print class".to_owned(),
//     //         ))?
//     //     }
//     // };
//     //
//     // 发生异常时，释放 native_print
//     // scopeguard::defer!(unsafe {
//     //     if is_exception.get() {
//     //         js_free(ctx.inner, native_print as _);
//     //         println!("js_free native_print");
//     //     }
//     // });

//     let mut native_print = Box::new(Cell::new(PrintClass { val: 0 }));

//     if args.len() > 0 {
//         let val = args[0].clone().to_int()?.value();
//         // unsafe { (*(native_print as *mut PrintClass)).val = val; }
//         native_print.get_mut().val = val;
//     }

//     // 获取 new_target 的 prototype
//     let proto = new_target
//         .get_property("prototype")
//         .ok_or(ez_quick_js::common::Error::GeneralError(
//             "JS_GetPropertyStr() is failed when get 'prototype'".to_owned(),
//         ))
//         .map_err(|err| {
//             // is_exception.set(true);
//             err
//         })?;

//     let cls_id = unsafe { PRINT_CLASS_ID };
//     let js_print_obj =
//         new_object_proto_class(ctx, &proto /*也可以设为 JS_NULL*/, cls_id).map_err(|err| {
//             // is_exception.set(true);
//             err
//         })?;

//     // 没有发生异常，则不释放 native_print
//     js_print_obj.set_opaque(Box::into_raw(native_print) as _);
//     // js_print_obj.set_opaque(native_print as _);

//     Ok(unsafe { js_print_obj.forget() })
// }

// /// PrintClass Constructor 的包装函数
// unsafe extern "C" fn js_printclass_constructor2(
//     ctx: *mut JSContext,
//     new_target: JSValue,
//     argc: ::std::os::raw::c_int,
//     argv: *mut JSValue,
// ) -> JSValue {
//     let ctx = Context::from_raw(ctx);
//     let new_target = JsValue::new(&ctx, new_target);

//     let rst = {
//         let args = {
//             let tmp = std::slice::from_raw_parts(argv, argc as usize);
//             let tmp = tmp
//                 .iter()
//                 .map(|val| JsValue::new(&ctx, *val))
//                 .collect::<Vec<JsValue>>();
//             tmp
//         };

//         js_printclass_constructor2_inner(&ctx, &new_target, &args)
//     };

//     let ret_val = if let Ok(val) = rst { val } else { JS_EXCEPTION };

//     new_target.forget();
//     ctx.forget();

//     ret_val
// }
