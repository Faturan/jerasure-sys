extern crate bindgen;

use std::path::PathBuf;
use std::{env, fs};

use bindgen::CargoCallbacks;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gc_out_dir = out_dir.join("gf-complete");
    fs::create_dir_all(&gc_out_dir).expect("Could not create gf-complete output dir.");
    let je_out_dir = out_dir.join("jerasure");
    fs::create_dir_all(&je_out_dir).expect("Could not create jerasure output dir.");

    let gc_dir_path = PathBuf::from("gf-complete")
        .canonicalize()
        .expect("cannot canonicalize path");
    let je_dir_path = PathBuf::from("jerasure")
        .canonicalize()
        .expect("cannot canonicalize path");

    let gc_headers_path = gc_dir_path.join("include");
    let je_headers_path = je_dir_path.join("include");
    let gc_lib_path = gc_out_dir.join("src/.libs");
    let je_lib_path = je_out_dir.join("src/.libs");

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
        .expect("could not spawn gf-complete `autoreconf`")
        .status
        .success()
    {
        panic!("could not autoreconf gf-complete");
    }

    if !std::process::Command::new(gc_dir_path.join("configure"))
        .current_dir(&gc_out_dir)
        .arg("--enable-static")
        .output()
        .expect("could not spawn gf-complete `configure`")
        .status
        .success()
    {
        panic!("could not configure gf-complete");
    }

    if !std::process::Command::new("make")
        .current_dir(&gc_out_dir)
        .output()
        .expect("could not spawn gf-complete `make`")
        .status
        .success()
    {
        panic!("could not make gf-complete");
    }

    if !std::process::Command::new("autoreconf")
        .current_dir(&je_dir_path)
        .arg("-i")
        .output()
        .expect("could not spawn jerasure `autoreconf`")
        .status
        .success()
    {
        panic!("could not autoreconf jerasure");
    }

    let arg1 = format!("LDFLAGS=-L{}", gc_lib_path.to_str().unwrap());
    let arg2 = format!("CPPFLAGS=-I{}", gc_headers_path.to_str().unwrap());

    if !std::process::Command::new(je_dir_path.join("configure"))
        .current_dir(&je_out_dir)
        .arg("--enable-static")
        .arg(&arg1)
        .arg(&arg2)
        .output()
        .expect("could not spawn jerasure `configure`")
        .status
        .success()
    {
        panic!("could not configure jerasure");
    }

    if !std::process::Command::new("make")
        .current_dir(&je_out_dir)
        .output()
        .expect("could not spawn jerasure `make`")
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

    let out_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
