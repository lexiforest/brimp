use web_runtime::{Browser, PageOptions};

fn page_with(html: &str) -> web_runtime::Page {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(html).unwrap();
    page
}

fn assert_js(page: &web_runtime::Page, expression: &str) {
    let result = page
        .eval(&format!("Number(Boolean({expression}))"))
        .unwrap()
        .to_number()
        .unwrap();
    assert_eq!(result, 1.0, "JavaScript assertion failed: {expression}");
}

#[test]
fn exposes_window_document_classes_and_stable_wrappers() {
    let page = page_with("<html><head></head><body><div id='box'>Hello</div></body></html>");

    assert_js(&page, "window === self");
    assert_js(&page, "window.window === window && window.self === window");
    assert_js(&page, "typeof addEventListener === 'function'");
    assert_js(&page, "typeof removeEventListener === 'function'");
    assert_js(&page, "typeof dispatchEvent === 'function'");
    assert_js(&page, "document === window.document");
    assert_js(&page, "document instanceof Document");
    assert_js(&page, "document instanceof Node");
    assert_js(&page, "document.body instanceof HTMLElement");
    assert_js(&page, "document.body instanceof Element");
    assert_js(&page, "document.body === document.body");
    assert_js(&page, "window instanceof Window");
    assert_js(&page, "new DOMRect(1, 2, 3, 4).right === 4");
    assert_js(&page, "document.body.style instanceof CSSStyleDeclaration");
    assert_js(
        &page,
        "document.querySelector('#box') === document.getElementById('box')",
    );
}

#[test]
fn exposes_url_and_url_search_params() {
    let page = page_with(
        "<html><head><base href='https://example.test/base/'></head>\
         <body><a id='link' href='../item?q=one'></a></body></html>",
    );

    assert_js(
        &page,
        "new URL('../item?q=one', 'https://example.test/base/').href === 'https://example.test/item?q=one'",
    );
    assert_js(
        &page,
        "new URL('https://user:pass@example.test:8443/a#b').origin === 'https://example.test:8443'",
    );
    assert_js(
        &page,
        "self.hasOwnProperty('URL') && self.hasOwnProperty('URLSearchParams')",
    );
    assert_js(
        &page,
        "(() => { const url = new URL('https://old.test:8000/'); url.host = 'new.test:9000'; return url.host === 'new.test:9000'; })()",
    );
    assert_js(
        &page,
        "(() => { const url = new URL('https://example.test/'); url.searchParams.append('a b', 'c+d'); return url.search === '?a+b=c%2Bd'; })()",
    );
    assert_js(
        &page,
        "(() => { const params = new URLSearchParams('b=2&a=1&a=3'); params.sort(); return params.toString() === 'a=1&a=3&b=2' && params.getAll('a').length === 2; })()",
    );
    assert_js(&page, "URL.canParse('/path', 'https://example.test/')");
    assert_js(&page, "URL.parse('not a url') === null");
    assert_js(
        &page,
        "document.querySelector('base') instanceof HTMLBaseElement && \
         document.getElementById('link') instanceof HTMLAnchorElement && \
         document.getElementById('link').href === 'https://example.test/item?q=one' && \
         document.getElementById('link').origin === 'https://example.test'",
    );
}

#[test]
fn exposes_dom_implementation_has_feature() {
    let page = page_with("<!doctype html><title>DOM implementation</title>");

    assert_js(
        &page,
        "document.implementation === document.implementation && \
         document.implementation instanceof DOMImplementation && \
         document.implementation.hasFeature() && \
         document.implementation.hasFeature('Core', '2.0')",
    );
}

#[test]
fn class_list_is_live_iterable_and_validates_tokens() {
    let page =
        page_with("<html><body><div id='box' class='  alpha alpha beta '></div></body></html>");

    assert_js(
        &page,
        r##"(() => {
            const box = document.getElementById("box");
            const list = box.classList;
            if (!(list instanceof DOMTokenList) || list !== box.classList) return false;
            if (list.length !== 2 || list[0] !== "alpha" || list.item(1) !== "beta") return false;
            if (String(list) !== "  alpha alpha beta ") return false;
            list.add("gamma", "alpha");
            list.remove("beta");
            if ([...list].join(",") !== "alpha,gamma") return false;
            if (!list.toggle("delta", true) || list.toggle("gamma", false)) return false;
            if (!list.replace("delta", "epsilon") || list.contains("delta")) return false;
            try { list.add("bad token"); return false; }
            catch (error) { return error instanceof DOMException && error.name === "InvalidCharacterError"; }
        })()"##,
    );
}

#[test]
fn element_queries_and_html_collections_are_scoped_and_live() {
    let page = page_with(
        "<html><body><section id='one'><div id='first' class='item common'></div></section>\
         <section id='two'><div id='second' class='item'></div></section></body></html>",
    );

    assert_js(
        &page,
        r##"(() => {
            const one = document.getElementById("one");
            const collection = one.getElementsByClassName("item common");
            if (!(collection instanceof HTMLCollection) || collection.length !== 1) return false;
            if (collection[0].id !== "first" || collection.namedItem("first") !== collection[0]) return false;
            const selected = one.querySelectorAll(".item");
            if (!(selected instanceof NodeList) || selected.length !== 1 || selected.map(node => node.id).join() !== "first") return false;
            if (one.querySelector("#second") !== null) return false;
            if (!collection[0].matches(".common") || collection[0].closest("section") !== one) return false;
            const added = document.createElement("div");
            added.className = "item common";
            one.appendChild(added);
            return collection.length === 2 && one.getElementsByTagName("div").length === 2;
        })()"##,
    );
}

#[test]
fn exposes_traversal_text_attributes_and_selectors() {
    let page =
        page_with("<html><head></head><body><div id='box' class='item'>Hello</div></body></html>");

    assert_js(&page, "document.nodeType === 9");
    assert_js(&page, "document.nodeName === '#document'");
    assert_js(&page, "document.documentElement.tagName === 'HTML'");
    assert_js(&page, "document.head.tagName === 'HEAD'");
    assert_js(&page, "document.body.tagName === 'BODY'");
    assert_js(
        &page,
        "document.body.parentNode === document.documentElement",
    );
    assert_js(&page, "document.body.childNodes.length === 1");
    assert_js(
        &page,
        "document.body.firstChild === document.body.lastChild",
    );
    assert_js(&page, "document.body.firstChild.nodeName === 'DIV'");
    assert_js(&page, "document.querySelectorAll('.item').length === 1");
    assert_js(
        &page,
        "document.querySelector('#box').textContent === 'Hello'",
    );
    assert_js(
        &page,
        "document.querySelector('#box').firstChild instanceof Text",
    );
    assert_js(
        &page,
        "document.querySelector('#box').getAttribute('class') === 'item'",
    );
    assert_js(
        &page,
        "document.querySelector('#box').getAttribute('missing') === null",
    );
}

#[test]
fn javascript_mutates_the_blitz_tree_directly() {
    let page = page_with("<html><head></head><body><div id='anchor'></div></body></html>");

    page.eval(
        r#"
        (() => {
            const body = document.body;
            const section = document.createElement("section");
            section.id = "created";
            section.className = "panel";
            section.setAttribute("data-test", "yes");
            section.appendChild(document.createTextNode("Created"));
            body.insertBefore(section, document.getElementById("anchor"));

            const anchor = document.getElementById("anchor");
            body.removeChild(anchor);
            section.textContent = "Updated";
            section.removeAttribute("data-test");
        })()
        "#,
    )
    .unwrap();

    let document = page.document();
    let created = document.query_selector("#created").unwrap().unwrap();
    let node = document.node(created).unwrap();
    assert_eq!(node.text_content(), "Updated");
    assert_eq!(node.attr(blitz_dom::local_name!("class")), Some("panel"));
    assert_eq!(node.attr(blitz_dom::LocalName::from("data-test")), None);
    assert!(document.query_selector("#anchor").unwrap().is_none());
}

#[test]
fn moving_existing_children_preserves_dom_order_and_identity() {
    let page = page_with(
        "<html><head></head><body><div id='one'></div><div id='two'></div></body></html>",
    );

    page.eval(
        r#"
        (() => {
            const body = document.body;
            const one = document.getElementById("one");
            const two = document.getElementById("two");
            body.insertBefore(two, one);
            if (body.firstChild !== two) throw new Error("insertBefore did not move the node");
            body.appendChild(two);
            if (body.lastChild !== two) throw new Error("appendChild did not move the node");
            one.id = "renamed";
            if (document.getElementById("one") !== null) throw new Error("old id remained live");
            if (document.getElementById("renamed") !== one) throw new Error("new id was not live");
            body.removeChild(one);
            if (document.getElementById("renamed") !== null) throw new Error("detached id remained live");
            if (one.parentNode !== null) throw new Error("removed node retained its parent");
        })()
        "#,
    )
    .unwrap();
}

#[test]
fn inner_html_and_style_are_live_blitz_mutations() {
    let page = page_with("<html><head></head><body><div id='box'></div></body></html>");

    page.eval(
        r#"
        (() => {
            const box = document.getElementById("box");
            box.innerHTML = '<span id="inside">Hi</span>';
            box.style.width = "120px";
            box.style.setProperty("padding", "10px");
        })()
        "#,
    )
    .unwrap();

    assert_js(
        &page,
        "document.getElementById('box').style === document.getElementById('box').style",
    );
    assert_js(
        &page,
        "document.getElementById('box').style.width === '120px'",
    );
    assert_js(
        &page,
        "document.getElementById('box').style.getPropertyValue('padding') === '10px'",
    );
    assert_js(
        &page,
        "document.getElementById('inside').textContent === 'Hi'",
    );

    let document = page.document();
    let box_id = document.get_element_by_id("box").unwrap();
    let box_node = document.node(box_id).unwrap();
    assert_eq!(box_node.text_content(), "Hi");
    assert!(
        box_node
            .attr(blitz_dom::local_name!("style"))
            .unwrap()
            .contains("width: 120px")
    );
}
