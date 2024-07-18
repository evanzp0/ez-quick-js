use ez_quick_js::{Context, Integer, Number, Runtime};

fn main() {
    let rt = Runtime::default();
    let ctx = Context::new(&rt);
    // let c = unsafe { js_new_float64(ctx.inner, f64) };
    let c = Number::new(&ctx, 3.15);
    let a: Integer = c.into();
    println!("{:?}", a);
}