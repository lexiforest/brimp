use std::path::PathBuf;

fn main() {
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }
    println!("cargo:rerun-if-env-changed=BRIMP_CURL_LIB_DIR");
    let directory = std::env::var_os("BRIMP_CURL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib"));
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS is missing");
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
