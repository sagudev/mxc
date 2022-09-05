use std::env;

fn main() {
    let statik = env::var("CARGO_FEATURE_STATIC").is_ok();

    let (taglib_include, taglib_lib) = taglib::get_taglib_paths();

    println!("cargo:rustc-link-search=native={}", taglib_lib.display());
    let ty = if statik { "static" } else { "dylib" };
    println!("cargo:rustc-link-lib={}=tag", ty);
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=z");
    }

    cxx_build::CFG.include_prefix = "";
    cxx_build::bridge("src/lib.rs") // returns a cc::Build
        .flag_if_supported("-fdiagnostics-color=always")
        .flag_if_supported("-Wno-unused-parameter")
        .file("src/tagg.cc")
        .include(taglib_include)
        .flag_if_supported("-std=c++11")
        .include("src")
        .compile("cxxtag");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/tagg.cc");
    println!("cargo:rerun-if-changed=src/tagg.h");
}
