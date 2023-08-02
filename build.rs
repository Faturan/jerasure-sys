extern crate bindgen;

use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use bindgen::CargoCallbacks;

fn cp_r(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let filetype = e.file_type()?;
        if filetype.is_dir() {
            cp_r(e.path(), dst.as_ref().join(e.file_name()))?;
        } else {
            fs::copy(e.path(), dst.as_ref().join(e.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gc_src = canonicalize("./gf-complete").expect("gf-complete submodule not found");
    let gc_dir_path = out_dir.join("gf-complete");

    cp_r(gc_src, &gc_dir_path).expect("could not copy gf-complete dir");

    let je_src = canonicalize("./jerasure").expect("jerasure submodule not found");
    let je_dir_path = out_dir.join("jerasure");

    cp_r(je_src, &je_dir_path).expect("could not copy jerasure dir");

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

    let out_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
