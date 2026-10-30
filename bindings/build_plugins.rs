use std::{fs, path::Path};

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create license subdirectory");
    for entry in fs::read_dir(source).expect("read license directory") {
        let path = entry.expect("license entry").path();
        let target = destination.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            fs::copy(&path, &target).expect("package native license");
        }
    }
}

pub fn package_plugins(destination: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("crates/runtime/js/plugins");
    let destination = Path::new(env!("CARGO_MANIFEST_DIR")).join(destination);
    fs::create_dir_all(&destination).expect("create plugin asset directory");
    for entry in fs::read_dir(&destination).expect("read packaged plugins") {
        let path = entry.expect("packaged plugin").path();
        if path.extension().is_some_and(|extension| extension == "js")
            && !source.join(path.file_name().unwrap()).exists()
        {
            fs::remove_file(path).expect("remove obsolete packaged plugin");
        }
    }
    for entry in fs::read_dir(&source).expect("read plugin sources") {
        let path = entry.expect("plugin source").path();
        if path.extension().is_some_and(|extension| extension == "js") {
            println!("cargo:rerun-if-changed={}", path.display());
            let source = fs::read_to_string(&path).expect("read plugin JavaScript");
            let source = if path.file_stem().is_some_and(|name| name == "extraction") {
                let bundle = root.join("assets/defuddle/0.19.3/index.full.js");
                println!("cargo:rerun-if-changed={}", bundle.display());
                format!(
                    "context => {{ {}; return ({source})(context); }}",
                    fs::read_to_string(bundle).expect("read Defuddle")
                )
            } else {
                source
            };
            fs::write(destination.join(path.file_name().unwrap()), source)
                .expect("package plugin JavaScript");
        }
    }
    let catalog = root.join("bindings/plugin-catalog.json");
    println!("cargo:rerun-if-changed={}", catalog.display());
    fs::copy(catalog, destination.join("catalog.json")).expect("package plugin catalog");
    let licenses = root.join("assets/defuddle/0.19.3/licenses");
    let output = destination.join("licenses");
    fs::create_dir_all(&output).expect("create license directory");
    for entry in fs::read_dir(licenses).expect("read plugin licenses") {
        let path = entry.expect("plugin license").path();
        println!("cargo:rerun-if-changed={}", path.display());
        fs::copy(&path, output.join(path.file_name().unwrap())).expect("package license");
    }
    for entry in
        fs::read_dir(root.join("crates/runtime/js/vendor")).expect("read primitive licenses")
    {
        let path = entry.expect("primitive notice").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "LICENSE")
        {
            println!("cargo:rerun-if-changed={}", path.display());
            fs::copy(&path, output.join(path.file_name().unwrap()))
                .expect("package primitive license");
        }
    }
    for (source, name) in [
        ("LICENSE", "brimp-LICENSE"),
        ("crates/lexbor/vendor/lexbor/LICENSE", "lexbor-LICENSE"),
        ("crates/lexbor/vendor/lexbor/NOTICE", "lexbor-NOTICE"),
        ("assets/defuddle/0.19.3/NOTICE.md", "defuddle-NOTICE.md"),
    ] {
        let path = root.join(source);
        println!("cargo:rerun-if-changed={}", path.display());
        fs::copy(path, output.join(name)).expect("package dependency notice");
    }
    let native = output.join("native");
    println!("cargo:rerun-if-env-changed=BRIMP_LICENSE_DIR");
    if native.exists() {
        fs::remove_dir_all(&native).expect("remove obsolete native licenses");
    }
    if let Some(source) = std::env::var_os("BRIMP_LICENSE_DIR") {
        let source = Path::new(&source);
        if !source.is_dir() {
            panic!("BRIMP_LICENSE_DIR is not a directory: {}", source.display());
        }
        println!("cargo:rerun-if-changed={}", source.display());
        copy_tree(source, &native);
    }
}
