use std::{fs, path::Path};

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
}
