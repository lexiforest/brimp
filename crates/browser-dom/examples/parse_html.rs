use browser_dom::BrowserDocument;

fn main() {
    let document =
        BrowserDocument::parse(r#"<html><body><div id="hello">Hello</div></body></html>"#);
    let hello = document.get_element_by_id("hello").unwrap();
    println!("{}", document.node(hello).unwrap().text_content());
}
