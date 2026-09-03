use browser_dom::BrowserDocument;

#[test]
fn serializes_html_void_and_non_void_elements() {
    let document = BrowserDocument::parse(
        "<html><head><script src='app.js'></script><title></title></head>\
         <body><!--kept--><div data-value='a&b\"c'></div><br><img src='https://example.test/x'></body></html>",
    );

    let html = document.outer_html();
    assert!(html.contains("<script src=\"app.js\"></script>"));
    assert!(html.contains("<title></title>"));
    assert!(html.contains("<div data-value=\"a&amp;b&quot;c\"></div>"));
    assert!(html.contains("<br>"));
    assert!(html.contains("<img src=\"https://example.test/x\">"));
    assert!(html.contains("<!--kept-->"));
    assert!(!html.contains(" />"));
}

#[test]
fn escapes_text_but_preserves_raw_text_element_contents() {
    let document = BrowserDocument::parse(
        "<html><head><style>.x > .y { color: red }</style></head>\
         <body><p>&lt;tag&gt; &amp; text</p></body></html>",
    );

    let html = document.outer_html();
    assert!(html.contains("<style>.x > .y { color: red }</style>"));
    assert!(html.contains("<p>&lt;tag&gt; &amp; text</p>"));
}
