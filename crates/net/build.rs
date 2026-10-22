use std::{env, path::PathBuf};

fn main() {
    if env::var_os("DOCS_RS").is_some() {
        return;
    }
    println!("cargo:rerun-if-env-changed=BRIMP_CURL_LIB_DIR");
    let directory = env::var_os("BRIMP_CURL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib"));
    let target = env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    let (filename, name) = match target.as_str() {
        "macos" => ("libcurl-impersonate.dylib", "curl-impersonate"),
        "windows" => ("libcurl-impersonate_imp.lib", "libcurl-impersonate_imp"),
        _ => ("libcurl-impersonate.so", "curl-impersonate"),
    };
    let library = directory.join(filename);
    assert!(
        library.is_file(),
        "{} not found; set BRIMP_CURL_LIB_DIR",
        library.display()
    );
    println!("cargo:rerun-if-changed={}", library.display());
    println!("cargo:rustc-link-search=native={}", directory.display());
    println!("cargo:rustc-link-lib=dylib={name}");
    println!("cargo:lib_dir={}", directory.display());
    if target != "windows" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", directory.display());
    }
}
