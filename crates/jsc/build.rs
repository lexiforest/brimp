use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=BRIMP_JSC_LIB_DIR");

    let library_dir = env::var_os("BRIMP_JSC_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../WebKit/WebKitBuild/Release")
        });
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS is missing");
    let library = match target_os.as_str() {
        "macos" => library_dir.join("JavaScriptCore.framework/JavaScriptCore"),
        "windows" => library_dir.join("JavaScriptCore.lib"),
        _ => library_dir.join("libJavaScriptCore.so"),
    };

    if !library.is_file() {
        panic!(
            "JavaScriptCore library not found at {} (set BRIMP_JSC_LIB_DIR)",
            library.display()
        );
    }
    let search_kind = if target_os == "macos" {
        "framework"
    } else {
        "native"
    };
    println!(
        "cargo:rustc-link-search={search_kind}={}",
        library_dir.display()
    );
    if target_os != "windows" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", library_dir.display());
    }
}
