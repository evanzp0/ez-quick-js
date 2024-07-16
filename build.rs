use std::{env, path::{Path, PathBuf}};

const LIB_NAME: &str = "quickjs";

fn exists(path: impl AsRef<Path>) -> bool {
    PathBuf::from(path.as_ref()).exists()
}

fn main() {
    let embed_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("deps/quickjs");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let code_dir = out_path.clone().join("quickjs");
    if exists(&code_dir) {
        std::fs::remove_dir_all(&code_dir).unwrap();
    }

    println!("cargo::rerun-if-changed={}", embed_path.to_str().unwrap());

    // let lib_name = {
    //     let os_type = std::env::consts::OS;
    // };
    
    copy_dir::copy_dir(embed_path, &code_dir)
        .expect("Could not copy quickjs directory");
    
    // std::fs::copy(
    //     embed_path.join("static-functions.c"),
    //     code_dir.join("static-functions.c"),
    // )
    // .expect("Could not copy static-functions.c");

    eprintln!("Compiling quickjs...");
    let quickjs_version =
        std::fs::read_to_string(code_dir.join("VERSION")).expect("failed to read quickjs version");
    cc::Build::new()
        .files(
            [
                "cutils.c",
                "libbf.c",
                "libregexp.c",
                "libunicode.c",
                "quickjs.c",
                // Custom wrappers.
                // "static-functions.c",
            ]
            .iter()
            .map(|f| code_dir.join(f)),
        )
        .define("_GNU_SOURCE", None)
        .define(
            "CONFIG_VERSION",
            format!("\"{}\"", quickjs_version.trim()).as_str(),
        )
        .define("CONFIG_BIGNUM", None)
        // The below flags are used by the official Makefile.
        .flag_if_supported("-Wchar-subscripts")
        .flag_if_supported("-Wno-array-bounds")
        .flag_if_supported("-Wno-format-truncation")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wundef")
        .flag_if_supported("-Wuninitialized")
        .flag_if_supported("-Wunused")
        .flag_if_supported("-Wwrite-strings")
        .flag_if_supported("-funsigned-char")
        // Below flags are added to supress warnings that appear on some
        // platforms.
        .flag_if_supported("-Wno-cast-function-type")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-enum-conversion")
        // cc uses the OPT_LEVEL env var by default, but we hardcode it to -O2
        // since release builds use -O3 which might be problematic for quickjs,
        // and debug builds only happen once anyway so the optimization slowdown
        // is fine.
        .opt_level(2)
        .compile(LIB_NAME);


    // println!("cargo::rustc-link-lib={}", LIB_NAME); // -l
    // println!("cargo::rustc-link-search={}", out_path); //-L
    
}