use std::fs;

use ez_quick_js::{
    ffi::{js_to_string, JSContext, JSValue, JS_EVAL_TYPE_GLOBAL, JS_UNDEFINED},
    function::new_cfunction,
    Context, Runtime,
};

fn main() {
    let file_name = "./examples/hello_world.js";
    let code = fs::read_to_string(file_name).unwrap();
    println!("{file_name}:\n{code}\n------------");

    let rt = Runtime::new(None);
    let ctx = &rt.create_context();

    add_global_print(ctx);

    println!("Eval script:");

    let _rst = ctx
        .eval(&code, file_name, JS_EVAL_TYPE_GLOBAL as i32)
        .unwrap();

    // println!("{:?}", _rst.to_string().unwrap().value());
}

fn add_global_print(ctx: &Context) {
    let global_obj = ctx.new_global_object();
    let console = ctx.new_object();

    console
        .set_property("log", new_cfunction(ctx, Some(js_print), "log", 1).unwrap())
        .unwrap();
    global_obj.set_property("console", console).unwrap();
    global_obj
        .set_property(
            "print",
            new_cfunction(ctx, Some(js_print), "log", 1).unwrap(),
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