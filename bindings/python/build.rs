#[path = "../build_plugins.rs"]
mod build_plugins;
fn main() {
    build_plugins::package_plugins("python/brimp/bundled");
    pyo3_build_config::add_extension_module_link_args();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        for key in [
            "DEP_BRIMP_RUNTIME_JSC_LIB_DIR",
            "DEP_BRIMP_RUNTIME_CURL_LIB_DIR",
        ] {
            let directory =
                std::env::var(key).expect("runtime must publish its native library paths");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
        }
    }
}
