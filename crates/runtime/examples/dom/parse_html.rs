use brimp_runtime::dom::BrowserDocument;

fn main() {
    let document =
        BrowserDocument::parse(r#"<html><body><div id="hello">Hello</div></body></html>"#);
    let hello = document.get_element_by_id("hello").unwrap();
    println!("{}", document.text(hello).unwrap().to_string_lossy());
}
