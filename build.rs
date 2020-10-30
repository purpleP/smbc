use pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    match pkg_config::probe_library("smbclient") {
        Ok(_) => {
            if cfg!(target_os = "macos") {
                println!("cargo:rustc-flags=-L /usr/local/lib -l smbclient");
            } else {
                println!("cargo:rustc-flags=-l smbclient");
            }
        }
        Err(e) => {
            println!("Error: SMB Client library not found!");
            panic!("{}", e);
        }
    };
    println!("cargo:rustc-link-lib=smbclient");
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header("wrapper.h")
        .clang_arg("-I/usr/include/samba-4.0")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
