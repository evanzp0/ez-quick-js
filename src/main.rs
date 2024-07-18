
use ez_quick_js::{Context, JsNumber, JsString, JsValue, Runtime};

fn main() {
    let rt = Runtime::default();
    let ctx = &Context::new(&rt);
    let s = JsString::new(&ctx, "abc");
    println!("{:?}", s);
    let v: JsValue = s.into();

    let c: JsString = v.to_string().unwrap();
    println!("c = {:?}", c);

    let a = JsNumber::new(ctx, 3_f64);
    let b: JsValue = a.into();
    let c: JsNumber = b.to_number().unwrap();
    println!("c = {:?}", c);
}