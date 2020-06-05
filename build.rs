use std::{env, path::PathBuf};

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        .header("external/src/errors.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn compile() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Build the actual errors set
    match std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "external/src/extensions.py {} /usr/share/xcb/*.xml",
            out_path.join("extensions.c").display()
        ))
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                println!("{}", String::from_utf8_lossy(&output.stderr));
                panic!("Failed to execute extensions.py:");
            }
        }
        Err(e) => panic!("{:?}", e),
    }

    cc::Build::new()
        .file("external/src/xcb_errors.c")
        .file(out_path.join("extensions.c"))
        .include("external/src")
        .compile("xcb_errors");
}

fn main() {
    println!("cargo:rustc-link-lib=xcb_errors");
    println!("cargo:rerun-if-changed=external/src/xcb_errors.c");
    println!("cargo:rerun-if-changed=external/src/xcb_errors.h");
    println!("cargo:rerun-if-changed=external/src/errors.h");

    compile();
    generate_bindings();
}
