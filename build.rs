extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for libraries for various distributions
    if std::path::Path::new("/lib64/libsolv.so.1").exists() {
        // openSUSE and SUSE
        println!("cargo:rustc-link-search=/lib64");
    } else {
        // ubuntu-latest: this gets used on github actions and has a different path
        println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
    }
    println!("cargo:rustc-link-lib=solv");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Compile the shim into a .a
    cc::Build::new()
        .file("src/solv_shim.c")
        .compile("solv_shim");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type(".*Pool")
        .allowlist_type("solv_knownid.*")
        .allowlist_var("SEARCH_STRING")
        .allowlist_function("pool_search")
        .allowlist_function("pool_setarch")
        .allowlist_function("pool_create")
        .allowlist_function("pool_free")
        .allowlist_function("repo_create")
        .allowlist_function("fopen")
        .allowlist_function("repo_add_solv")
        .allowlist_function("fclose")
        .allowlist_function("solvable_lookup_str")
        .allowlist_function("repodata_dir2str")
        .allowlist_function("cnf.*")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
