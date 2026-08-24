use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=BRIMP_JSC_FRAMEWORK_DIR");

    if env::var_os("CARGO_CFG_TARGET_OS").as_deref() != Some("macos".as_ref()) {
        panic!("jsc-sys currently supports the macOS JavaScriptCore framework only");
    }

    let framework_dir = env::var_os("BRIMP_JSC_FRAMEWORK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../WebKit/WebKitBuild/Release")
        });
    let framework = framework_dir.join("JavaScriptCore.framework/JavaScriptCore");

    if !framework.is_file() {
        panic!(
            "JavaScriptCore framework not found at {} (set BRIMP_JSC_FRAMEWORK_DIR)",
            framework.display()
        );
    }

    println!(
        "cargo:rustc-link-search=framework={}",
        framework_dir.display()
    );
    println!("cargo:rustc-link-lib=framework=JavaScriptCore");
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        framework_dir.display()
    );
}
