use ez_quick_js::{Context, Integer, Number, Runtime};

fn main() {
    let rt = Runtime::default();
    let ctx = Context::new(&rt);
    let a = Integer::new(&ctx, 54);
    // let c = unsafe { js_new_float64(ctx.inner, f64) };
    // let c = Number::new(&ctx, 3f64);
    let c: Number = a.into();
    println!("{:?}", c);
}