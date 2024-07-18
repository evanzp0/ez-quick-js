
use ez_quick_js::{Context, JsNumber, JsString, Runtime};

fn main() {
    let rt = Runtime::default();
    let ctx = &Context::new(&rt);
    let s = JsString::new(&ctx, "abc");
    println!("{:?}", s);
    let v = s.to_value();

    let c: JsString = v.to_string().unwrap();
    println!("c = {:?}", c);

    let a = JsNumber::new(ctx, 3_f64);
    let b = a.to_value();
    let c: JsNumber = b.to_number().unwrap();
    println!("c = {:?}", c);
}