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

    let gc_headers_path = gc_dir_path.join("include/gf_complete.h");
    let gc_headers_path_str = gc_headers_path
        .to_str()
        .expect("Path is not a valid string");
    let je_headers_path = je_dir_path.join("include/jerasure.h");
    let je_headers_path_str = je_headers_path
        .to_str()
        .expect("Path is not a valid string");

    let gc_lib_path = gc_dir_path.join("src/.libs/");
    let je_lib_path = je_dir_path.join("src/.libs/");

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", gc_lib_path.to_str().unwrap());
    println!("cargo:rustc-link-search={}", je_lib_path.to_str().unwrap());

    println!("cargo:rustc-env=LD_LIBRARY_PATH={};{}", gc_lib_path.to_str().unwrap(), je_lib_path.to_str().unwrap());

    let gc_lib_name = "gf_complete";
    let je_lib_name = "Jerasure";

    println!("cargo:rustc-link-lib={}", gc_lib_name);
    println!("cargo:rustc-link-lib={}", je_lib_name);

    // Tell cargo to invalidate the built crate whenever the header changes.
    println!("cargo:rerun-if-changed={}", gc_headers_path_str);
    println!("cargo:rerun-if-changed={}", je_headers_path_str);

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

    let arg1 = format!("LDFLAGS=-L{}/src/.libs/", gc_dir_path.to_str().unwrap());
    let arg2 = format!("CPPFLAGS=-I{}/include", gc_dir_path.to_str().unwrap());

    if !std::process::Command::new("./configure")
        .current_dir(&je_dir_path)
        .args(vec![&arg1, &arg2])
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

    let carg = format!("-I{}/include", gc_dir_path.to_str().unwrap());

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(&carg)
        // The input header we would like to generate
        // bindings for.
        .header(gc_headers_path_str)
        .header(je_headers_path_str)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
