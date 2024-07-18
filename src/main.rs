
use ez_quick_js::{Context, Runtime, String as JsString};

fn main() {
    let rt = Runtime::default();
    let ctx = Context::new(&rt);
    let s = JsString::new(&ctx, "abc");

    println!("{:?}", s);
}