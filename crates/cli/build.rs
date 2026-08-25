use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=BRIMP_CURL_LIB_DIR");
    let directory = std::env::var_os("BRIMP_CURL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib"));
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", directory.display());
}
