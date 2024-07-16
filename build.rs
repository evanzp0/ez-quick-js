use std::process::Command;

const LIB_NAME: &str = "quickjs";

fn main() {

    // println!("cargo:rustc-link-lib=static={}", LIB_NAME);

    Command::new("ls")
        .output()
        .unwrap();
}