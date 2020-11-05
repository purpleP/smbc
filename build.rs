use pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let smbclient_lib = pkg_config::probe_library("smbclient").unwrap();
    println!("cargo:rustc-link-lib=smbclient");
    println!("cargo:rerun-if-changed=wrapper.h");
    let include_args = smbclient_lib
        .include_paths
        .iter()
        .flat_map(|path| path.to_str())
        .map(|raw_path| format!("-I{}", raw_path));
    let libs_args = smbclient_lib
        .libs
        .iter()
        .map(|l| format!("-l{}", l));
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header("wrapper.h")
        .clang_args(include_args)
        .clang_args(libs_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
