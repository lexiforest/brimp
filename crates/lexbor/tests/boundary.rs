use brimp_lexbor::{Document, Error, declarations};
const HTML: &str = "http://www.w3.org/1999/xhtml";

#[test]
fn canonical_dom_ownership_and_detachment() {
    let doc = Document::parse("<!doctype html><main><p id='first'>a &amp; b</p><p>last</p></main>")
        .unwrap();
    let main = doc.select(doc.root(), "main").unwrap()[0];
    let first = doc.select(main, "#first").unwrap()[0];
    let text = doc.children(first).unwrap()[0];
    doc.set_text(first, "replacement").unwrap();
    assert_eq!(doc.text(text).unwrap(), "a & b");
    assert!(doc.parent(text).unwrap().is_none());
    doc.detach(first).unwrap();
    assert_eq!(doc.select(main, "p").unwrap().len(), 1);
    assert_eq!(doc.text(first).unwrap(), "replacement");
    doc.set_attribute(first, "data-note", "'\" & λ").unwrap();
    assert_eq!(
        doc.attribute(first, "data-note").unwrap().as_deref(),
        Some("'\" & λ")
    );
    assert!(doc.matches(first, "p#first[data-note]").unwrap());
    doc.insert(main, first, None).unwrap();
    assert_eq!(doc.select(main, "p").unwrap()[1], first);
    assert!(matches!(
        doc.insert(first, main, None),
        Err(Error::Hierarchy)
    ));
    let other = Document::new().unwrap();
    assert!(matches!(other.text(first), Err(Error::ForeignNode)));
    drop(doc);
    assert!(matches!(other.text(first), Err(Error::ForeignNode)));
}

#[test]
fn script_suspension_survives_mutations_and_nested_parsing() {
    let doc = Document::new().unwrap();
    let html = "<main><!--<script>inert</script>--><script title='>'>first</script><p id='later'>later</p><template><script>inert</script></template><script>second</script><i>last</i></main>";
    let mut parser = doc.parser(html, true).unwrap();
    let script = parser.resume().unwrap().unwrap();
    assert_eq!(doc.text(script).unwrap(), "first");
    assert!(doc.select(doc.root(), "#later").unwrap().is_empty());
    let main = doc.select(doc.root(), "main").unwrap()[0];
    let fragment = doc.fragment(main, "<b>λ &amp; nested</b>").unwrap();
    doc.insert(main, fragment, None).unwrap();
    assert_eq!(
        doc.text(doc.select(main, "b").unwrap()[0]).unwrap(),
        "λ & nested"
    );
    doc.detach(script).unwrap();
    let second = parser.resume().unwrap().unwrap();
    assert_eq!(doc.text(second).unwrap(), "second");
    assert_eq!(doc.select(main, "#later").unwrap().len(), 1);
    assert!(doc.select(main, "i").unwrap().is_empty());
    assert!(parser.resume().unwrap().is_none());
    assert!(parser.resume().unwrap().is_none());
    assert_eq!(doc.select(main, "i").unwrap().len(), 1);
    assert_eq!(doc.text(script).unwrap(), "first");
}

#[test]
fn contexts_templates_namespaces_and_serialization_use_lexbor() {
    let doc = Document::parse(
        "<table></table><template><p>template</p></template><svg viewBox='0 0 1 1'><circle/></svg>",
    )
    .unwrap();
    let table = doc.select(doc.root(), "table").unwrap()[0];
    let fragment = doc.fragment(table, "<tr><td>cell</td></tr>").unwrap();
    doc.insert(table, fragment, None).unwrap();
    assert_eq!(doc.select(table, "tbody > tr > td").unwrap().len(), 1);
    let template = doc.select(doc.root(), "template").unwrap()[0];
    let content = doc.template_content(template).unwrap().unwrap();
    assert_eq!(doc.select(content, "p").unwrap().len(), 1);
    assert!(doc.select(doc.root(), "p").unwrap().is_empty());
    let svg = doc.select(doc.root(), "svg").unwrap()[0];
    assert_eq!(doc.namespace(svg).unwrap(), "http://www.w3.org/2000/svg");
    assert_eq!(
        doc.attribute(svg, "viewBox").unwrap().as_deref(),
        Some("0 0 1 1")
    );
    let body = doc.select(doc.root(), "body").unwrap()[0];
    let node = doc.create_element("div", HTML, "").unwrap();
    let comment = doc.create_comment("comment").unwrap();
    doc.insert(node, comment, None).unwrap();
    let text = doc.create_text("<>&").unwrap();
    doc.insert(node, text, None).unwrap();
    doc.insert(body, node, None).unwrap();
    assert_eq!(
        doc.serialize(node, false).unwrap(),
        "<div><!--comment-->&lt;&gt;&amp;</div>"
    );
    assert!(doc.select(body, "[[[").is_err());
    let gradient = doc
        .create_element("linearGradient", "http://www.w3.org/2000/svg", "s")
        .unwrap();
    assert_eq!(doc.local_name(gradient).unwrap(), "linearGradient");
    assert_eq!(doc.name(gradient).unwrap(), "s:linearGradient");
    assert_eq!(doc.prefix(gradient).unwrap(), "s");
    assert!(
        doc.serialize(gradient, false)
            .unwrap()
            .contains("linearGradient")
    );
}

#[test]
fn declarations_preserve_priority_custom_values_and_recover_from_errors() {
    let properties =
        declarations("color: red; padding: 2px !important; --text: 'a;b'; invalid; width: 5px;")
            .unwrap();
    assert!(properties.contains(&("padding".into(), "2px".into(), true)));
    assert!(
        properties
            .iter()
            .any(|(name, value, _)| name == "--text" && value.contains("a;b"))
    );
    assert!(
        properties
            .iter()
            .any(|(name, value, _)| name == "width" && value == "5px")
    );
    assert!(!properties.iter().any(|(name, _, _)| name == "invalid"));
}

#[test]
fn parser_retains_arena_and_cancellation_drops_cleanly() {
    for _ in 0..100 {
        let doc = Document::new().unwrap();
        let mut parser = doc
            .parser("<script>pause</script><p>later</p>", true)
            .unwrap();
        assert!(parser.resume().unwrap().is_some());
        drop(doc);
        assert!(parser.resume().unwrap().is_none());
    }
    for _ in 0..100 {
        let doc = Document::new().unwrap();
        let mut parser = doc
            .parser("<script>pause</script><p>unparsed</p>", true)
            .unwrap();
        assert!(parser.resume().unwrap().is_some());
        drop(parser);
    }
}

#[test]
fn selectors_scope_has_and_boolean_attributes_are_native() {
    let doc = Document::parse("<main><article class='a'><b></b></article><article><i></i></article><script async defer></script></main>").unwrap();
    let main = doc.select(doc.root(), "main").unwrap()[0];
    assert_eq!(
        doc.select(main, ":scope > article:has(> b)").unwrap().len(),
        1
    );
    assert_eq!(
        doc.select(main, ":scope > article:not(:has(i))")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(doc.select(main, "article, article.a").unwrap().len(), 2);
    let script = doc.select(main, "script").unwrap()[0];
    assert_eq!(doc.attribute(script, "async").unwrap().as_deref(), Some(""));
    assert_eq!(doc.attribute(script, "defer").unwrap().as_deref(), Some(""));
    assert!(doc.attribute(script, "absent").unwrap().is_none());
}
