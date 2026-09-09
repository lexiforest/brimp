use std::{env, path::PathBuf};

fn main() {
    configure_jsc();
    configure_network();
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let dir = env::var("BRIMP_CURL_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".into());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");
    }
}

fn configure_jsc() {
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

fn configure_network() {
    if env::var_os("DOCS_RS").is_some() {
        return;
    }
    println!("cargo:rerun-if-env-changed=BRIMP_CURL_LIB_DIR");
    let directory = env::var_os("BRIMP_CURL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib"));
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS is missing");
    let (library_name, link_name) = match target_os.as_str() {
        "macos" => ("libcurl-impersonate.dylib", "curl-impersonate"),
        "windows" => ("libcurl-impersonate_imp.lib", "libcurl-impersonate_imp"),
        _ => ("libcurl-impersonate.so", "curl-impersonate"),
    };
    let library = directory.join(library_name);
    if !library.is_file() {
        panic!(
            "libcurl-impersonate was not found at {}; set BRIMP_CURL_LIB_DIR",
            library.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", directory.display());
    println!("cargo:rustc-link-lib=dylib={link_name}");
}
