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
    let gc_lib_path = gc_dir_path.join("src/.libs");
    let je_lib_path = je_dir_path.join("src/.libs");

    println!(
        "cargo:rustc-link-search=native={}",
        gc_lib_path.to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        je_lib_path.to_str().unwrap()
    );

    let gc_lib_name = "gf_complete";
    let je_lib_name = "Jerasure";

    println!("cargo:rustc-link-lib=static={}", gc_lib_name);
    println!("cargo:rustc-link-lib=static={}", je_lib_name);

    println!("cargo:rerun-if-changed={}", "wrapper.h");

    if !std::process::Command::new("autoreconf")
        .current_dir(&gc_dir_path)
        .arg("-i")
        .output()
        .expect("could not spawn `autoreconf`")
        .status
        .success()
    {
        panic!("could not autoreconf gf-complete");
    }

    if !std::process::Command::new("./configure")
        .current_dir(&gc_dir_path)
        .arg("--enable-static")
        .output()
        .expect("could not spawn `./configure`")
        .status
        .success()
    {
        panic!("could not configure gf-complete");
    }

    if !std::process::Command::new("make")
        .current_dir(&gc_dir_path)
        .output()
        .expect("could not spawn `make`")
        .status
        .success()
    {
        panic!("could not make gf-complete");
    }

    if !std::process::Command::new("autoreconf")
        .current_dir(&je_dir_path)
        .arg("-i")
        .output()
        .expect("could not spawn `autoreconf`")
        .status
        .success()
    {
        panic!("could not compile jerasure");
    }

    let arg1 = format!("LDFLAGS=-L{}", gc_lib_path.to_str().unwrap());
    let arg2 = format!("CPPFLAGS=-I{}", gc_headers_path.to_str().unwrap());

    if !std::process::Command::new("./configure")
        .current_dir(&je_dir_path)
        .arg("--enable-static")
        .arg(&arg1)
        .arg(&arg2)
        .output()
        .expect("could not spawn `./configure`")
        .status
        .success()
    {
        panic!("could not configure jerasure");
    }

    if !std::process::Command::new("make")
        .current_dir(&je_dir_path)
        .output()
        .expect("could not spawn `make`")
        .status
        .success()
    {
        panic!("could not make jerasure");
    }

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
