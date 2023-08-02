extern crate bindgen;

use std::env;
use std::path::PathBuf;

use bindgen::CargoCallbacks;

fn main() {
    let gc_dir_path = PathBuf::from("gf-complete")
        .canonicalize()
        .expect("cannot canonicalize path");
    let je_dir_path = PathBuf::from("jerasure")
        .canonicalize()
        .expect("cannot canonicalize path");

    let gc_headers_path = gc_dir_path.join("include");
    let je_headers_path = je_dir_path.join("include");

    let gc_dst = autotools::Config::new(&gc_dir_path)
        .reconf("-if")
        .enable_static()
        .build();

    println!("cargo:rustc-link-search=native={}/lib", gc_dst.display());

    let je_dst = autotools::Config::new(&je_dir_path)
        .reconf("-if")
        .enable_static()
        .ldflag(format!("-L{}/lib", gc_dst.to_str().unwrap()))
        .cflag(format!("-I{}", gc_headers_path.to_str().unwrap()))
        .build();

    println!("cargo:rustc-link-search=native={}/lib", je_dst.display());
    println!("cargo:rustc-link-lib=static=gf_complete");
    println!("cargo:rustc-link-lib=static=Jerasure");
    println!("cargo:rerun-if-changed=wrapper.h");

    let gc_clang_arg = format!("-I{}", gc_headers_path.to_str().unwrap());
    let je_clang_arg = format!("-I{}", je_headers_path.to_str().unwrap());

    let bindings = bindgen::Builder::default()
        .clang_arg(&gc_clang_arg)
        .clang_arg(&je_clang_arg)
        .header("wrapper.h")
        .parse_callbacks(Box::new(CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
