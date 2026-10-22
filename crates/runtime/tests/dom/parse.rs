use brimp_runtime::dom::BrowserDocument;

const HTML: &str = "<html><body><div id='hello' class='greeting'>Hello</div></body></html>";
#[test]
fn parses_html_into_a_traversable_canonical_document() {
    let document = BrowserDocument::parse(HTML);
    assert_eq!(document.kind(document.root()).unwrap(), 9);
    let html = document.query_selector("html").unwrap().unwrap();
    assert_eq!(document.local_name(html).unwrap(), "html");
    let body = document.query_selector("body").unwrap().unwrap();
    assert_eq!(document.parent(body).unwrap(), Some(html));
}
#[test]
fn exposes_elements_and_text_from_the_canonical_tree() {
    let document = BrowserDocument::parse(HTML);
    let by_id = document.get_element_by_id("hello").unwrap();
    assert_eq!(document.query_selector("#hello").unwrap(), Some(by_id));
    assert_eq!(
        document.query_selector_all(".greeting").unwrap(),
        vec![by_id]
    );
    assert_eq!(document.local_name(by_id).unwrap(), "div");
    assert_eq!(document.text(by_id).unwrap(), "Hello");
    let text = document.children(by_id).unwrap()[0];
    assert_eq!(document.kind(text).unwrap(), 3);
    assert_eq!(document.text(text).unwrap(), "Hello");
}
