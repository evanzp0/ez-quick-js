use ez_quick_js::{Context, Integer, Runtime};

fn main() {
    let rt = Runtime::default();
    let ctx = Context::new(&rt);

    let a = Integer::new(&ctx, 12);

    println!("{:?}", a);
}