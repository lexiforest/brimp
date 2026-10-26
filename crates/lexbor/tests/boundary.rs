use brimp_lexbor::{Document, DomString, Error, declarations};
const HTML: &str = "http://www.w3.org/1999/xhtml";

#[test]
fn text_code_units_survive_native_storage_cloning_and_replacement() {
    let doc = Document::new().unwrap();
    let parent = doc.create_element("div", HTML, "").unwrap();
    let units = [0xd800, 0, 0xd83e, 0xdd80, 0xdc00];
    let text = doc.create_text(DomString::from_utf16(&units)).unwrap();
    let comment = doc.create_comment(DomString::from_utf16(&units)).unwrap();
    doc.insert(parent, text, None).unwrap();
    doc.insert(parent, comment, None).unwrap();
    for node in [text, comment, parent, doc.clone_node(parent, true).unwrap()] {
        assert_eq!(doc.text(node).unwrap().to_utf16(), units);
    }
    assert_eq!(doc.serialize(text, false).unwrap().to_utf16(), units);
    let next = [0xdc00, 0x78, 0xd800];
    doc.set_text(parent, DomString::from_utf16(&next)).unwrap();
    assert_eq!(doc.text(parent).unwrap().to_utf16(), next);
    assert_eq!(doc.children(parent).unwrap().len(), 1);
    assert_eq!(doc.text(text).unwrap().to_utf16(), units);
    doc.set_text(comment, DomString::from_utf16(&next)).unwrap();
    assert_eq!(doc.text(comment).unwrap().to_utf16(), next);
}

#[test]
fn attribute_code_units_survive_native_storage_cloning_and_serialization() {
    let doc = Document::parse("<input>").unwrap();
    let input = doc.select(doc.root(), "input").unwrap()[0];
    let units = [0xd800, 0, 0xd83e, 0xdd80, 0xdc00, 0x22, 0x26];
    doc.set_attribute(input, "name", DomString::from_utf16(&units))
        .unwrap();
    doc.set_attribute_ns(input, "urn:test", "t:key", DomString::from_utf16(&units))
        .unwrap();
    for node in [input, doc.clone_node(input, true).unwrap()] {
        assert_eq!(
            doc.attribute(node, "name").unwrap().unwrap().to_utf16(),
            units
        );
        let attributes = doc.attributes(node).unwrap();
        assert_eq!(attributes.len(), 2);
        assert!(
            attributes
                .iter()
                .all(|attribute| attribute.value.to_utf16() == units)
        );
        let expected: Vec<_> = "<input name=\""
            .encode_utf16()
            .chain(units[..5].iter().copied())
            .chain("&quot;&amp;\" t:key=\"".encode_utf16())
            .chain(units[..5].iter().copied())
            .chain("&quot;&amp;\">".encode_utf16())
            .collect();
        assert_eq!(doc.serialize(node, false).unwrap().to_utf16(), expected);
    }
}

#[test]
fn document_modes_and_doctype_identifiers_survive_cloning() {
    for (html, quirks, public, system) in [
        ("<!doctype html>", false, "", ""),
        ("<!doctype other>", true, "", ""),
        (
            "<!doctype html SYSTEM 'about:legacy-compat'>",
            false,
            "",
            "about:legacy-compat",
        ),
        ("<!doctype html PUBLIC 'public'>", false, "public", ""),
        (
            "<!doctype html PUBLIC 'public' 'system'>",
            false,
            "public",
            "system",
        ),
        (
            "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN'>",
            true,
            "-//W3C//DTD HTML 4.01 Transitional//EN",
            "",
        ),
        (
            "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN' 'system'>",
            false,
            "-//W3C//DTD HTML 4.01 Transitional//EN",
            "system",
        ),
    ] {
        let doc = Document::parse(html).unwrap();
        assert_eq!(doc.document_quirks(doc.root()).unwrap(), quirks);
        let doctype = doc.children(doc.root()).unwrap()[0];
        for node in [doctype, doc.clone_node(doctype, true).unwrap()] {
            assert_eq!(doc.doctype_public_id(node).unwrap(), public);
            assert_eq!(doc.doctype_system_id(node).unwrap(), system);
            assert!(doc.document_quirks(node).is_err());
        }
        let cloned_doc = doc.clone_node(doc.root(), true).unwrap();
        assert_eq!(doc.document_quirks(cloned_doc).unwrap(), quirks);
        let copied = doc.create_document().unwrap();
        assert!(!doc.document_quirks(copied).unwrap());
        doc.copy_document_mode(doc.root(), copied).unwrap();
        assert_eq!(doc.document_quirks(copied).unwrap(), quirks);
        doc.set_document_no_quirks(copied).unwrap();
        assert!(!doc.document_quirks(copied).unwrap());
        assert!(doc.set_document_no_quirks(doctype).is_err());
        assert!(doc.copy_document_mode(doctype, copied).is_err());
        assert!(doc.doctype_public_id(doc.root()).is_err());
        let foreign = Document::new().unwrap();
        assert!(matches!(
            foreign.document_quirks(doc.root()),
            Err(Error::ForeignNode)
        ));
        assert!(matches!(
            foreign.doctype_public_id(doctype),
            Err(Error::ForeignNode)
        ));
    }
}

#[test]
fn canonical_dom_ownership_and_detachment() {
    let doc = Document::parse("<!doctype html><main><p id='first'>a &amp; b</p><p>last</p></main>")
        .unwrap();
    let main = doc.select(doc.root(), "main").unwrap()[0];
    let first = doc.select(main, "#first").unwrap()[0];
    let text = doc.children(first).unwrap()[0];
    doc.set_text(first, "replacement").unwrap();
    assert_eq!(doc.text(text).unwrap().to_string_lossy(), "a & b");
    assert!(doc.parent(text).unwrap().is_none());
    doc.detach(first).unwrap();
    assert_eq!(doc.select(main, "p").unwrap().len(), 1);
    assert_eq!(doc.text(first).unwrap().to_string_lossy(), "replacement");
    doc.set_attribute(first, "data-note", "'\" & λ").unwrap();
    assert_eq!(
        doc.attribute(first, "data-note")
            .unwrap()
            .as_ref()
            .and_then(|value| value.as_str()),
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
    assert_eq!(doc.text(script).unwrap().to_string_lossy(), "first");
    assert!(doc.select(doc.root(), "#later").unwrap().is_empty());
    let main = doc.select(doc.root(), "main").unwrap()[0];
    let fragment = doc.fragment(main, "<b>λ &amp; nested</b>").unwrap();
    doc.insert(main, fragment, None).unwrap();
    assert_eq!(
        doc.text(doc.select(main, "b").unwrap()[0])
            .unwrap()
            .to_string_lossy(),
        "λ & nested"
    );
    doc.detach(script).unwrap();
    let second = parser.resume().unwrap().unwrap();
    assert_eq!(doc.text(second).unwrap().to_string_lossy(), "second");
    assert_eq!(doc.select(main, "#later").unwrap().len(), 1);
    assert!(doc.select(main, "i").unwrap().is_empty());
    assert!(parser.resume().unwrap().is_none());
    assert!(parser.resume().unwrap().is_none());
    assert_eq!(doc.select(main, "i").unwrap().len(), 1);
    assert_eq!(doc.text(script).unwrap().to_string_lossy(), "first");
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
        doc.attribute(svg, "viewBox")
            .unwrap()
            .as_ref()
            .and_then(|value| value.as_str()),
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
        doc.serialize(node, false).unwrap().to_string_lossy(),
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
            .to_string_lossy()
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
    assert_eq!(
        doc.attribute(script, "async")
            .unwrap()
            .as_ref()
            .and_then(|value| value.as_str()),
        Some("")
    );
    assert_eq!(
        doc.attribute(script, "defer")
            .unwrap()
            .as_ref()
            .and_then(|value| value.as_str()),
        Some("")
    );
    assert!(doc.attribute(script, "absent").unwrap().is_none());
}

#[test]
fn namespace_attributes_are_distinct_and_preserve_case() {
    let doc = Document::parse(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'><a xlink:href='old'/></svg>",
    )
    .unwrap();
    let svg = doc.select(doc.root(), "svg").unwrap()[0];
    doc.set_attribute_ns(svg, "urn:a", "a:Key", "one").unwrap();
    doc.set_attribute_ns(svg, "urn:b", "b:Key", "two").unwrap();
    doc.set_attribute_ns(svg, "urn:a", "other:Key", "updated")
        .unwrap();
    let attrs = doc.attributes(svg).unwrap();
    let a = attrs
        .iter()
        .find(|a| a.namespace.as_deref() == Some("urn:a"))
        .unwrap();
    assert_eq!(a.name, "a:Key");
    assert_eq!(a.local_name, "Key");
    assert_eq!(a.prefix.as_deref(), Some("a"));
    assert_eq!(a.value.as_str(), Some("updated"));
    assert!(
        attrs
            .iter()
            .find(|a| a.name == "viewBox")
            .unwrap()
            .namespace
            .is_none()
    );
    doc.remove_attribute_ns(svg, "urn:a", "Key").unwrap();
    assert!(
        doc.attributes(svg)
            .unwrap()
            .iter()
            .any(|a| a.namespace.as_deref() == Some("urn:b"))
    );
    let clone = doc.clone_node(svg, true).unwrap();
    assert_eq!(
        doc.attributes(clone).unwrap().len(),
        doc.attributes(svg).unwrap().len()
    );
}
#[test]
fn stylesheet_rules_and_incremental_writer_keep_owned_buffers() {
    let rules =
        brimp_lexbor::stylesheet("@media screen { b { color:red } } p {color: blue !important}")
            .unwrap();
    assert_eq!(
        rules.iter().map(|r| (r.depth, r.kind)).collect::<Vec<_>>(),
        vec![(0, 4), (1, 1), (0, 1)]
    );
    assert_eq!(rules[1].prelude, "b");
    assert_eq!(
        declarations(" \n color: red !important").unwrap(),
        vec![("color".into(), "red".into(), true)]
    );
    let doc = Document::new().unwrap();
    let mut writer = doc.writer_at(doc.root()).unwrap();
    writer.write("<!doctype html><p id='sp");
    assert!(writer.resume().unwrap().is_none());
    writer.write("lit'>hello<script>run</script><b>end</b>");
    let script = writer.resume().unwrap().unwrap();
    assert_eq!(doc.text(script).unwrap().to_string_lossy(), "run");
    writer.write("<i>nested</i>");
    assert!(writer.resume().unwrap().is_none());
    assert!(doc.select(doc.root(), "b").unwrap().is_empty());
    assert!(writer.resume().unwrap().is_none());
    writer.finish().unwrap();
    drop(writer);
    assert_eq!(doc.select(doc.root(), "#split i + b").unwrap().len(), 1);
}

#[test]
fn parser_form_owners_follow_moves_without_leaking_into_clones_or_fragments() {
    let doc =
        Document::parse("<div id=root><table><form id=f><tr><td><input id=i></table></form></div>")
            .unwrap();
    let find = |selector| doc.select(doc.root(), selector).unwrap()[0];
    let root = find("#root");
    let table = find("table");
    let form = find("#f");
    let input = find("#i");
    assert_eq!(doc.parser_form(input).unwrap(), Some(form));
    assert_eq!(doc.parser_form(root).unwrap(), None);
    doc.insert(root, table, None).unwrap();
    assert_eq!(doc.parser_form(input).unwrap(), Some(form));
    for deep in [false, true] {
        let copy = doc.clone_node(input, deep).unwrap();
        assert_eq!(doc.parser_form(copy).unwrap(), None);
    }
    let copy = doc.clone_node(table, true).unwrap();
    assert_eq!(
        doc.parser_form(doc.select(copy, "input").unwrap()[0])
            .unwrap(),
        None
    );
    let fragment = doc
        .fragment(root, "<table><form><tr><td><input></table>")
        .unwrap();
    assert_eq!(
        doc.parser_form(doc.select(fragment, "input").unwrap()[0])
            .unwrap(),
        None
    );
    doc.set_attribute_ns(input, "urn:test", "form", "foreign")
        .unwrap();
    doc.remove_attribute(input, "class").unwrap();
    assert_eq!(doc.parser_form(input).unwrap(), Some(form));
    doc.detach(form).unwrap();
    assert_eq!(doc.parser_form(input).unwrap(), None);
    doc.insert(table, form, None).unwrap();
    assert_eq!(doc.parser_form(input).unwrap(), None);
    assert!(matches!(
        Document::new().unwrap().parser_form(input),
        Err(Error::ForeignNode)
    ));
}

#[test]
fn parser_form_owners_are_recorded_in_secondary_documents_and_streaming_writers() {
    let doc = Document::new().unwrap();
    let root = doc.create_document().unwrap();
    let mut parser = doc
        .parser_at(root, "<table><form><tr><td><input></table></form>", false)
        .unwrap();
    assert!(parser.resume().unwrap().is_none());
    let form = doc.select(root, "form").unwrap()[0];
    let input = doc.select(root, "input").unwrap()[0];
    assert_eq!(doc.parser_form(input).unwrap(), Some(form));
    let written = doc.create_document().unwrap();
    let mut writer = doc.writer_at(written).unwrap();
    writer.write("<table><form id=f><tr><td><input id=i><script>pause</script>");
    assert!(writer.resume().unwrap().is_some());
    let form = doc.select(written, "form").unwrap()[0];
    let input = doc.select(written, "input").unwrap()[0];
    assert_eq!(doc.parser_form(input).unwrap(), Some(form));
    writer.write("</table></form><input id=outside>");
    assert!(writer.resume().unwrap().is_none());
    assert_eq!(
        doc.parser_form(doc.select(written, "#outside").unwrap()[0])
            .unwrap(),
        None
    );
}

#[test]
fn parser_ignores_a_form_pointer_separated_by_an_intervening_script() {
    let doc = Document::new().unwrap();
    let mut parser = doc
        .parser(
            "<table><form><tr><td><script>pause</script><input></table>",
            true,
        )
        .unwrap();
    assert!(parser.resume().unwrap().is_some());
    let form = doc.select(doc.root(), "form").unwrap()[0];
    doc.detach(form).unwrap();
    assert!(parser.resume().unwrap().is_none());
    let input = doc.select(doc.root(), "input").unwrap()[0];
    assert_eq!(doc.parser_form(input).unwrap(), None);
}
