use std::path::{Path, PathBuf};

fn first_existing(directory: &Path, names: &[&str]) -> Option<PathBuf> {
    names
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.exists())
}

fn main() {
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }
    println!("cargo:rerun-if-env-changed=BRIMP_CURL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=BRIMP_CURL_STATIC");
    let directory = std::env::var_os("BRIMP_CURL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib"));
    let static_library = first_existing(&directory, &["libcurl-impersonate.a"]);
    #[cfg(target_os = "macos")]
    let dynamic_library = first_existing(&directory, &["libcurl-impersonate.dylib"]);
    #[cfg(not(target_os = "macos"))]
    let dynamic_library = first_existing(&directory, &["libcurl-impersonate.so"]);
    println!("cargo:rustc-link-search=native={}", directory.display());
    if std::env::var_os("BRIMP_CURL_STATIC").is_some_and(|value| value == "1")
        && static_library.is_some()
    {
        println!("cargo:rustc-link-lib=static=curl-impersonate");
    } else if dynamic_library.is_some() {
        println!("cargo:rustc-link-lib=dylib=curl-impersonate");
    } else if static_library.is_some() {
        println!("cargo:rustc-link-lib=static=curl-impersonate");
    } else {
        panic!(
            "libcurl-impersonate was not found in {}; set BRIMP_CURL_LIB_DIR",
            directory.display()
        );
    }
}
