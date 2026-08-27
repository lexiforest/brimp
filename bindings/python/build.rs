fn main() {
    pyo3_build_config::add_extension_module_link_args();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS is missing");
    if target_os != "windows" {
        let curl_directory =
            std::env::var("BRIMP_CURL_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".into());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{curl_directory}");
        if let Ok(jsc_directory) = std::env::var("BRIMP_JSC_LIB_DIR") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{jsc_directory}");
        }
    }
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}
