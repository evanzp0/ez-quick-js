
use ez_quick_js::{Context, JsInteger, JsValue, Runtime};

fn main() {
    let rt = Runtime::default();
    let ctx = &Context::new(&rt);
    // let s = JsString::new(&ctx, "abc");
    // println!("{:?}", s);
    let a = JsInteger::new(ctx, 12);
    let b: JsValue = a.into();
    println!("b = {:?}", b);

    let c: JsInteger = b.try_into().unwrap();
    println!("c = {:?}", c);
}