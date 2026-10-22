use std::{env, path::PathBuf};

fn main() {
    use std::path::Path;
    fn sources(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).expect("vendored Lexbor source directory") {
            let path = entry.expect("Lexbor directory entry").path();
            if path.is_dir() {
                sources(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "c") {
                files.push(path);
            }
        }
    }
    let root = Path::new("vendor/lexbor/source");
    let mut files = Vec::new();
    for module in ["core", "dom", "ns", "tag", "html", "css", "selectors"] {
        sources(&root.join("lexbor").join(module), &mut files);
    }
    let port = if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        "windows_nt"
    } else {
        "posix"
    };
    sources(&root.join("lexbor/ports").join(port), &mut files);
    files.sort();
    println!("cargo:rerun-if-changed=vendor/lexbor");
    println!("cargo:rerun-if-changed=native/lexbor.c");
    println!("cargo:rerun-if-changed=native/lexbor.h");
    cc::Build::new()
        .include(root)
        .define("LEXBOR_STATIC", None)
        .define("_POSIX_C_SOURCE", "200809L")
        .std("c11")
        .warnings(false)
        .files(files)
        .file("native/lexbor.c")
        .compile("brimp_lexbor");
}
