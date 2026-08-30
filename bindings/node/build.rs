fn main() {
    napi_build::setup();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let directory =
            std::env::var("BRIMP_CURL_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".into());
        println!("cargo:rustc-link-arg=-Wl,-headerpad_max_install_names");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}
