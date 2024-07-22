use std::fs;

use ez_quick_js::{ffi::{js_to_string, JSContext, JSValue, JS_EVAL_TYPE_GLOBAL, JS_PROP_C_W_E, JS_UNDEFINED}, Context, JsInteger, JsValue, Runtime};

fn main() {
    let file_name = "./examples/show_js_point.js";
    let code = &fs::read_to_string(file_name).unwrap();

    let rt = Runtime::new(None);
    let ctx = &rt.create_context();
    let global_obj = ctx.get_global_object();

    add_global_print(ctx);

    ctx.eval(code, file_name, JS_EVAL_TYPE_GLOBAL as i32).unwrap();
    let js_show_point_fn = global_obj.get_property("show_point").unwrap();
    
    let js_point = create_js_point(ctx, 1, 2);
}

fn create_js_point<'a>(ctx: &'a Context, pt_x: i32, pt_y: i32) -> JsValue<'a> {
    let js_point_obj = ctx.new_object().unwrap();
    js_point_obj.define_property("x", ctx.get_int(pt_x), JS_PROP_C_W_E as i32).unwrap();
    js_point_obj.define_property("y", ctx.get_int(pt_y), JS_PROP_C_W_E as i32).unwrap();
 
    js_point_obj
}

unsafe extern "C" fn point_multiple(
    ctx: *mut JSContext,
    this_val: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    let ctx = &Context::from_raw(ctx);

    println!();

    JS_UNDEFINED
}

fn get_point_from_js(ctx: &Context, this_obj: &JsValue) -> Point {
    let x = this_obj.get_property("x").unwrap().to_int().unwrap().value();
    let y = this_obj.get_property("y").unwrap().to_int().unwrap().value();

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
        .set_property(
            "print",
            ctx.get_cfunction(js_print, "log", 1).unwrap(),
        )
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

struct Point {
    x: i32,
    y: i32,
}