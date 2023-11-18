extern crate bindgen;

use std::env;
use std::path::PathBuf;

#[cfg(feature = "__c_api")]
fn gen_c_api() {
    if std::env::var("_CBINDGEN_IS_RUNNING").is_ok() {
        return;
    }

    fn get_build_profile_name() -> String {
        // The profile name is always the 3rd last part of the path (with 1 based indexing).
        // e.g. /code/core/target/cli/build/my-build-info-9f91ba6f99d7a061/out
        let out_dir = std::env::var("OUT_DIR")
            .expect("OUT_DIR is not set, cannot determine build profile, aborting");
        out_dir
            .split(std::path::MAIN_SEPARATOR)
            .nth_back(3)
            .expect("Cannot determine build profile, aborting")
            .to_string()
    }

    /// Find the location of the `target/` directory. Note that this may be
    /// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
    /// variable.
    fn target_dir() -> PathBuf {
        if let Ok(target) = env::var("CARGO_TARGET_DIR") {
            PathBuf::from(target)
        } else {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
                .join(format!("../target/{}", get_build_profile_name()))
        }
    }

    extern crate cbindgen;
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{package_name}.h"))
        .display()
        .to_string();

    let parse_expand_features_vec = vec![
        #[cfg(feature = "__c_api")]
        "__c_api",
        #[cfg(feature = "boolean-c-api")]
        "boolean-c-api",
        #[cfg(feature = "shortint-c-api")]
        "shortint-c-api",
        #[cfg(feature = "high-level-c-api")]
        "high-level-c-api",
        #[cfg(feature = "boolean")]
        "boolean",
        #[cfg(feature = "shortint")]
        "shortint",
        #[cfg(feature = "integer")]
        "integer",
    ];

    let parse_expand_vec = if parse_expand_features_vec.is_empty() {
        vec![]
    } else {
        vec![package_name.as_str()]
    };

    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_config(cbindgen::Config::from_root_or_default(crate_dir))
        .with_parse_expand(&parse_expand_vec)
        .with_parse_expand_features(&parse_expand_features_vec)
        .generate()
        .unwrap()
        .write_to_file(output_file);
}

fn main() {
    #[cfg(feature = "__c_api")]
    gen_c_api();

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=/opt/xilinx/xrt/lib");

    // Tell cargo to tell rustc to link the system xrt libraries
    // shared library.
    println!("cargo:rustc-link-lib=xrt_coreutil");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/opt/xilinx/xrt/include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
