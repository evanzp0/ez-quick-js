use std::cell::Cell;
use std::ffi::c_int;
use std::ptr::null_mut;
use std::fs;

use anyhow::Error;
use ez_quick_js::ffi::{
    Find_Export_Entry, JSCFunctionEnum_JS_CFUNC_constructor, JSClassDef, JSClassID, JSModuleDef,
    JSRuntime, JS_AtomToString, JS_Find_Loaded_Module, JS_FreeValue, JS_GetModuleName,
    JS_GetOpaque, JS_GetOpaque2, JS_GetPropertyStr, JS_GetRuntime, JS_IsException, JS_NewAtomLen,
    JS_NewCFunction2, JS_NewClass, JS_NewInt32, JS_NewObject, JS_NewObjectProtoClass,
    JS_PromiseResult, JS_PromiseState, JS_SetClassProto, JS_SetConstructor, JS_SetModuleExport,
    JS_SetOpaque, JS_SetPropertyFunctionList, JS_ToInt32, JS_ToStr, JS_EVAL_TYPE_GLOBAL,
    JS_EVAL_TYPE_MODULE, JS_TAG_INT,
};
use ez_quick_js::function::{add_module_export, call_js_function, new_class_id, C_FUNC_DEF, C_GET_SET_DEF};
use ez_quick_js::{
    ffi::{JSCFunctionListEntry, JSContext, JSValue},
    Context, Runtime,
};
use ez_quick_js::{JsInteger, JsModuleDef, JsValue, JS_EXCEPTION, JS_UNDEFINED};
use once_cell::sync::Lazy;

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
static PRINT_CLASS_ID: Lazy<JSClassID> = Lazy::new(|| {
    let mut tmp = 0;
    new_class_id(&mut tmp)
});

unsafe extern "C" fn js_print_cls_finalizer(_rt: *mut JSRuntime, val: JSValue) {
    let native_print = JS_GetOpaque(val, *PRINT_CLASS_ID) as *mut PrintClass;

    println!("js_print_cls_finalizer run: {}", (*native_print).val);

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
///     它就是我们后面将 js_print_constructor 注册到全局对象上的那个 JS Function
unsafe extern "C" fn js_print_constructor(
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
        if JS_IsException(proto) {
            return JS_EXCEPTION;
        }

        // 用 ctor.proto 对应的 shape 生成一个新 JS 实例对象，该对象的 class_id 为 print_class_id, prototype 为 proto
        let obj = JS_NewObjectProtoClass(
            ctx,
            proto, /* 也可以设为 JS_NULL */
            *PRINT_CLASS_ID,
        );
        JS_FreeValue(ctx, proto);

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
        let tmp = JS_GetOpaque2(ctx, this_val, *PRINT_CLASS_ID) as *mut PrintClass;
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
        let tmp = JS_GetOpaque2(ctx, this_val, *PRINT_CLASS_ID) as *mut PrintClass;
        if tmp == null_mut() {
            return JS_EXCEPTION;
        }
        tmp
    };

    // 获取 print.val 并转成 JSValue 类型
    let val = native_print_val_getter(native_print.as_ref().unwrap());
    let js_val = JS_NewInt32(ctx, val);

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
    let native_print: *mut PrintClass = JS_GetOpaque2(ctx, this_val, *PRINT_CLASS_ID) as _;
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

unsafe extern "C" fn init_module_inner(ctx: *mut JSContext, m: *mut JSModuleDef) -> c_int {
    println!("PRINT_CLASS_ID : {}", *PRINT_CLASS_ID);

    // 创建 PrintClass 类，并在 rt->class_array 中注册它
    JS_NewClass(JS_GetRuntime(ctx), *PRINT_CLASS_ID, &PRINT_CLASS_DEF);

    // 生成 prototype
    let js_print_proto = JS_NewObject(ctx);

    // 为 prototype 设置方法（这些方法会被实例化的子对象继承）
    JS_SetPropertyFunctionList(
        ctx,
        js_print_proto,
        JS_PRINT_FUNCS.as_ptr(),
        JS_PRINT_FUNCS.len() as i32,
    );

    // 生成名为 Point 的 JSCFunction 对象(父类是 JSObject )，该对象包装了 Point 的构造器( js_print_constructor )
    let js_print_ctor = JS_NewCFunction2(
        ctx,
        Some(js_print_constructor),
        PRINT_CLASS_DEF.class_name,
        2,
        JSCFunctionEnum_JS_CFUNC_constructor,
        0,
    );

    /* set proto.constructor and ctor.prototype */
    // 1. 将 js_point_ctor 的 prototype 值设为 js_print_proto
    // 2. 将 js_print_proto 的 construct 值设为 js_print_ctor
    JS_SetConstructor(ctx, js_print_ctor, js_print_proto);

    // 将 ctx->class_proto[PRINT_CLASS_ID] 指针指向 js_print_proto
    JS_SetClassProto(ctx, *PRINT_CLASS_ID, js_print_proto);

    // 将 js_print_ctor 添加到 module 的 "Print" 这个 属性上
    // 注意：必须要先执行 JS_AddModuleExport() 然后才能执行 JS_SetModuleExport()，
    //      否则 JS_SetModuleExport() 导出对象时，找不到导出的属性名会报错
    JS_SetModuleExport(ctx, m, PRINT_CLASS_DEF.class_name, js_print_ctor);

    0
}

/// 创建模块并导出对象
fn init_module<'a>(ctx: &'a Context, module_name: &str) -> Result<JsModuleDef<'a>, Error> {
    // 创建模块，并初始化模块内本地对象
    let m = ctx.new_module(module_name, Some(init_module_inner))?;

    // 导出 tab (JS_FUNC_LIST) 列表中同名的本地对象
    add_module_export(ctx, &m, PRINT_CLASS_DEF.class_name)?;

    Ok(m)
}

#[test]
fn test_module() -> Result<(), Error> {
    // load js script
    let file_name = "./tests/test_module.js";
    let code = &fs::read_to_string(file_name)?;
    // println!("{code}");

    let rt = Runtime::new(None);
    let ctx = &rt.create_context();

    let md = init_module(ctx, "_G")?;
    unsafe {
        let md_name_atom = JS_GetModuleName(ctx.inner, md.raw_value() as _);
        println!("{}", md_name_atom);

        let m_str = {
            let val = JS_AtomToString(ctx.inner, md_name_atom);
            JS_ToStr(ctx.inner, val)
        };
        println!("{}", m_str);
    }

    let rst_promise = ctx.eval(
        code,
        "ff_module",
        (JS_EVAL_TYPE_GLOBAL | JS_EVAL_TYPE_MODULE) as i32,
    )?;
    assert!(rst_promise.is_object());

    let state = unsafe { JS_PromiseState(ctx.inner, *rst_promise.raw_value()) };
    println!("state = {}", state);

    let s = unsafe { JS_PromiseResult(ctx.inner, *rst_promise.raw_value()) };
    unsafe {
        println!("promise_result = {:p}", s.u.ptr);
    }

    // assert -----------------
    let ff_module = unsafe {
        let ff_atom = JS_NewAtomLen(ctx.inner, b"ff_module\0".as_ptr() as _, 9);
        JS_Find_Loaded_Module(ctx.inner, ff_atom)
    };
    assert!(ff_module != null_mut());
    unsafe {

        let ret_val_entry = {
            let ret_atom = JS_NewAtomLen(ctx.inner, b"ret_val\0".as_ptr() as _, 7);
            Find_Export_Entry(ctx.inner, ff_module, ret_atom)
        };

        let export_name = {
            let val = JS_AtomToString(ctx.inner, (*ret_val_entry).export_name);
            let v = JS_ToStr(ctx.inner, val);
            JS_FreeValue(ctx.inner, val);
            v
        };
        assert_eq!("ret_val", export_name);

        let ret_val = JsValue::new(ctx, *(*(*ret_val_entry).u.local.var_ref).pvalue);
        let ret_val = ret_val.to_int().unwrap().value();
        assert_eq!(40, ret_val);
    }

    let ff_module = JsModuleDef::new(ctx, ff_module);
    {
        // let default_entry = {
        //     let ret_atom = JS_NewAtomLen(ctx.inner, b"default\0".as_ptr() as _, 7);
        //     let atom = JsAtom::new(ctx, ret_atom);
        //     let name = atom.to_str();
        //     assert_eq!("default", name);
        //     Find_Export_Entry(ctx.inner, ff_module, ret_atom)
        // };

        // {
        //     let export_name = {
        //         let val = JS_AtomToString(ctx.inner, (*default_entry).export_name);
        //         JS_ToStr(ctx.inner, val)
        //     };
        //     assert_eq!("default", export_name);

        //     let default_val = JsValue::new(ctx, *(*(*default_entry).u.local.var_ref).pvalue);
        //     let tmp = default_val.to_string().unwrap();
        //     let default_str = tmp.value();
        //     assert_eq!("evan", default_str);
        // }

        let entry = ff_module.find_export_entry("default");
        assert!(entry.is_some());

        let val = entry.unwrap().export_value().to_string().unwrap();
        assert_eq!("evan", val.value());
    }

    {
        let entry = ff_module.find_export_entry("add_one");
        assert!(entry.is_some());
        let js_func = entry.unwrap().export_value();
        let param = JsInteger::new(ctx, 2);
        let rst = call_js_function(ctx, &js_func, None, &vec![&param.to_value()]);
        assert!(rst.is_ok());
        assert_eq!(3, rst.unwrap().to_int().unwrap().value());
    }

    Ok(())
}
