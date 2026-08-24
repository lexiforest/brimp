use blitz_dom::{NodeData, local_name};
use browser_dom::BrowserDocument;

const HTML: &str = r#"
    <html>
    <body>
        <div id="hello" class="greeting">Hello</div>
    </body>
    </html>
"#;

#[test]
fn parses_html_into_a_traversable_blitz_document() {
    let document = BrowserDocument::parse(HTML);
    assert!(matches!(document.root().data, NodeData::Document));

    let html_id = document.query_selector("html").unwrap().unwrap();
    let html = document.node(html_id).unwrap();
    assert!(html.data.is_element_with_tag_name(&local_name!("html")));

    let body_id = document.query_selector("body").unwrap().unwrap();
    assert_eq!(document.node(body_id).unwrap().parent, Some(html_id));
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

    let element = document.node(by_id).unwrap();
    assert!(element.data.is_element_with_tag_name(&local_name!("div")));
    assert_eq!(element.text_content(), "Hello");

    let text = element
        .children
        .iter()
        .find_map(|id| document.node(*id).and_then(|node| node.text_data()))
        .unwrap();
    assert_eq!(text.content, "Hello");
}
