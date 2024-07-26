use std::fs;

use ez_quick_js::{
    ffi::{
        JSContext, JSValue, JS_GetPropertyStr, JS_NewInt32, JS_ToI32, JS_ToStr,
        JS_EVAL_TYPE_GLOBAL, JS_PROP_C_W_E,
    },
    function::call_js_function,
    Context, JsValue, Runtime, JS_UNDEFINED,
};

fn main() {
    let file_name = "./examples/show_js_point.js";
    let code = &fs::read_to_string(file_name).unwrap();

    let rt = Runtime::new(None);
    let ctx = &rt.create_context();
    let global_obj = ctx.get_global_object();

    add_global_print(ctx);

    ctx.eval(code, file_name, JS_EVAL_TYPE_GLOBAL as i32)
        .unwrap();
    let js_show_point_fn = global_obj.get_property("show_point").unwrap();
    let js_point_obj = create_js_point(ctx, 2, 3);
    let rst = call_js_function(ctx, &js_show_point_fn, None, &vec![&js_point_obj]).unwrap();
    let point = get_point_from_js(ctx.inner, *rst.raw_value());
    println!("return : {:?}", point);
}

fn create_js_point<'a>(ctx: &'a Context, pt_x: i32, pt_y: i32) -> JsValue<'a> {
    let js_point_obj = ctx.new_object().unwrap();
    js_point_obj
        .define_property("x", ctx.get_int(pt_x), JS_PROP_C_W_E as i32)
        .unwrap();
    js_point_obj
        .define_property("y", ctx.get_int(pt_y), JS_PROP_C_W_E as i32)
        .unwrap();
    js_point_obj
        .set_property(
            "multiple",
            ctx.get_cfunction(point_multiple, "multiple", 0).unwrap(),
        )
        .unwrap();

    js_point_obj
}

fn get_point_from_js(ctx: *mut JSContext, this_obj: JSValue) -> Point {
    let p_name_x = "x\0";
    let p_name_y = "y\0";

    let x = unsafe {
        let val = JS_GetPropertyStr(ctx, this_obj, p_name_x.as_ptr() as _);
        JS_ToI32(ctx, val)
    };
    let y = unsafe {
        let val = JS_GetPropertyStr(ctx, this_obj, p_name_y.as_ptr() as _);
        JS_ToI32(ctx, val)
    };

    Point { x, y }
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

unsafe extern "C" fn point_multiple(
    ctx: *mut JSContext,
    this_val: JSValue,
    _argc: ::std::os::raw::c_int,
    _argv: *mut JSValue,
) -> JSValue {
    let val = {
        let point = get_point_from_js(ctx, this_val);
        let val = point.x * point.y;
        JS_NewInt32(ctx, val)
    };

    val
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

        let str = JS_ToStr(ctx, *item);
        print!("{str}");
    }

    println!();

    JS_UNDEFINED
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}
