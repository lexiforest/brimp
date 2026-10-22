use std::env;

fn main() {
    // Relay native SDK metadata to the language bindings and link runtime tests/examples.
    for (key, metadata) in [
        ("DEP_JAVASCRIPTCORE_JSC_LIB_DIR", "jsc_lib_dir"),
        ("DEP_CURL_IMPERSONATE_LIB_DIR", "curl_lib_dir"),
    ] {
        let directory = env::var(key).expect("native dependency must publish its SDK path");
        println!("cargo:{metadata}={directory}");
        if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
        }
    }
}
