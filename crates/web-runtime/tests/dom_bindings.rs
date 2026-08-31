use web_runtime::{Browser, PageOptions};

fn page_with(html: &str) -> web_runtime::Page {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(html).unwrap();
    page
}

#[test]
fn installs_the_window_chain_and_enforces_web_idl_invocation_shape() {
    let page = page_with("<html><body><div></div></body></html>");

    assert_js(
        &page,
        r#"(() => {
            if (Object.getPrototypeOf(window) !== Window.prototype) return false;
            if (Object.getPrototypeOf(Window.prototype) !== WindowProperties.prototype) return false;
            if (Object.getPrototypeOf(WindowProperties.prototype) !== EventTarget.prototype) return false;
            if (!(window instanceof Window) || !(window instanceof WindowProperties) ||
                !(window instanceof EventTarget)) return false;
            if (window.constructor !== Window || Object.prototype.toString.call(window) !== "[object Window]") {
                return false;
            }

            for (const constructor of [Window, WindowProperties, Navigator, History]) {
                try { new constructor(); return false; }
                catch (error) { if (!(error instanceof TypeError) || error.message !== "Illegal constructor") return false; }
                try { constructor(); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
            }

            const calls = [
                () => Document.prototype.querySelector.call({}, "div"),
                () => EventTarget.prototype.addEventListener.call({}, "probe", () => {}),
                () => Object.getOwnPropertyDescriptor(Navigator.prototype, "userAgent").get.call({}),
                () => Object.getOwnPropertyDescriptor(Window.prototype, "innerWidth").get.call({}),
                () => Object.getOwnPropertyDescriptor(Storage.prototype, "length").get.call({}),
                () => Object.getOwnPropertyDescriptor(Event.prototype, "isTrusted").get.call({}),
            ];
            for (const call of calls) {
                try { call(); return false; }
                catch (error) { if (!(error instanceof TypeError) || error.message !== "Illegal invocation") return false; }
            }

            const querySelector = Document.prototype.querySelector;
            const userAgent = Object.getOwnPropertyDescriptor(Navigator.prototype, "userAgent").get;
            return querySelector.name === "querySelector" && querySelector.length === 1 &&
                userAgent.name === "get userAgent" && userAgent.length === 0 &&
                Function.prototype.toString.call(querySelector) === "function querySelector() { [native code] }" &&
                Function.prototype.toString.call(userAgent) === "function get userAgent() { [native code] }";
        })()"#,
    );
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
    assert_js(
        &page,
        "window.parent === window && window.top === window && window.opener === null",
    );
    assert_js(&page, "typeof addEventListener === 'function'");
    assert_js(&page, "typeof removeEventListener === 'function'");
    assert_js(&page, "typeof dispatchEvent === 'function'");
    assert_js(&page, "document === window.document");
    assert_js(
        &page,
        "self.Node === Node && self.Document === Document && self.Element === Element && self.HTMLElement === HTMLElement",
    );
    assert_js(
        &page,
        "Node.ELEMENT_NODE === 1 && document.TEXT_NODE === 3 && new Text('abc') instanceof CharacterData",
    );
    assert_js(&page, "document instanceof Document");
    assert_js(&page, "document instanceof Node");
    assert_js(&page, "document.body instanceof HTMLElement");
    assert_js(&page, "document.body instanceof Element");
    assert_js(&page, "document.body === document.body");
    assert_js(&page, "window instanceof Window");
    assert_js(&page, "new DOMRect(1, 2, 3, 4).right === 4");
    assert_js(
        &page,
        "(() => { const box = document.getElementById('box'); return box.getClientRects().length === 1 && (box.scrollIntoView(), true); })()",
    );
    assert_js(&page, "document.body.style instanceof CSSStyleDeclaration");
    assert_js(
        &page,
        "document.querySelector('#box') === document.getElementById('box')",
    );
    assert_js(
        &page,
        "(() => { const box = document.getElementById('box'); box.title = 'tip'; box.autofocus = true; box.tabIndex = 7; box.classList = 'one two'; return box.title === 'tip' && box.hasAttribute('autofocus') && box.tabIndex === 7 && box.className === 'one two'; })()",
    );
}

#[test]
fn focus_tracks_the_active_element_and_dispatches_events() {
    let page =
        page_with("<html><body><input id='first'><textarea id='second'></textarea></body></html>");
    assert_js(
        &page,
        r#"(() => {
            const events = [];
            const first = document.getElementById("first");
            const second = document.getElementById("second");
            first.addEventListener("focus", () => events.push("first-focus"));
            first.addEventListener("blur", () => events.push("first-blur"));
            second.addEventListener("focus", () => events.push("second-focus"));
            first.focus();
            first.value = "one";
            second.focus();
            second.value = "two";
            return document.activeElement === second && first.value === "one" &&
                second.value === "two" && events.join(",") === "first-focus,first-blur,second-focus";
        })()"#,
    );
}

#[test]
fn exposes_live_elements_by_name_and_animation_timing() {
    let page = page_with("<html><head></head><body><input name='item'></body></html>");

    assert_js(
        &page,
        r##"(() => {
            const nodes = document.getElementsByName("item");
            if (!(nodes instanceof NodeList) || nodes.length !== 1 || nodes.item(0) !== nodes[0]) {
                return false;
            }
            const input = document.createElement("input");
            input.setAttribute("name", "item");
            document.body.appendChild(input);
            if (nodes.length !== 2 || [...nodes][1] !== input) return false;
            input.remove();
            return nodes.length === 1;
        })()"##,
    );
    assert_js(
        &page,
        "performance instanceof Performance && performance.now() >= 0 && Number.isFinite(performance.timeOrigin)",
    );
    assert_js(
        &page,
        "typeof requestAnimationFrame === 'function' && typeof cancelAnimationFrame === 'function'",
    );
    assert_js(
        &page,
        "document.hidden === false && document.visibilityState === 'visible'",
    );
}

#[test]
fn exposes_live_element_attributes() {
    let page = page_with("<html><body><div id='box' data-value='one'></div></body></html>");

    assert_js(
        &page,
        r#"(() => {
            const box = document.getElementById("box");
            const attributes = box.attributes;
            if (!(attributes instanceof NamedNodeMap) || attributes !== box.attributes || attributes.length !== 2) {
                return false;
            }
            const data = attributes.getNamedItem("data-value");
            if (!(data instanceof Attr) || !(data instanceof Node) || data !== attributes[1] ||
                data.name !== "data-value" || data.localName !== "data-value" ||
                data.namespaceURI !== null || data.prefix !== null || data.value !== "one" ||
                data.ownerElement !== box || data.nodeType !== Node.ATTRIBUTE_NODE) return false;
            box.setAttribute("title", "tip");
            if (attributes.length !== 3 || attributes.title.value !== "tip") return false;
            data.value = "two";
            if (box.getAttribute("data-value") !== "two") return false;
            const removed = attributes.removeNamedItem("data-value");
            if (removed !== data || removed.ownerElement !== null || attributes.length !== 2) return false;
            if (Object.getOwnPropertyNames(attributes).join(",") !== "0,1,id,title") return false;
            if (!box.toggleAttribute("hidden") || !box.hasAttribute("hidden")) return false;
            if (box.toggleAttribute("hidden") || box.hasAttribute("hidden")) return false;
            return box.getAttributeNames().join(",") === "id,title";
        })()"#,
    );
}

#[test]
fn exposes_css_supports_and_escape() {
    let page = page_with("<html><body></body></html>");

    assert_js(&page, "CSS.supports('display', 'block')");
    assert_js(&page, "CSS.supports('display: block')");
    assert_js(&page, "CSS.supports('(display: block)')");
    assert_js(&page, "!CSS.supports('not-a-property', 'anything')");
    assert_js(
        &page,
        "!CSS.supports('display', 'definitely-not-a-display-value')",
    );
    assert_js(&page, "CSS.supports('selector(div > span)')");
    assert_js(&page, "!CSS.supports('selector(div >> span)')");
    assert_js(
        &page,
        r#"CSS.escape("a b") === "a\\ b" && CSS.escape("0a") === "\\30 a" && CSS.escape("\0") === "\uFFFD""#,
    );
    assert_js(
        &page,
        r#"(() => {
            const style = document.body.style;
            style.fontFamily = "serif";
            style.fontFamily = "invalid()";
            if (style.fontFamily !== "serif") return false;
            style.font = "16px sans-serif";
            const parsed = style.font;
            style.font = "16px invalid()";
            return style.font === parsed;
        })()"#,
    );
}

#[test]
fn inline_font_family_uses_stylo_cssom_serialization() {
    let page = page_with("<html><body><div id='target'></div></body></html>");

    assert_js(
        &page,
        r#"(() => {
            const style = document.getElementById("target").style;
            const values = [
                "serif",
                "'family;with:semicolon'",
                "'family:with:colon'",
                "'字體 家族'",
                "'quoted family', sans-serif",
            ];
            for (const value of values) {
                style.fontFamily = value;
                if (style.fontFamily === "") return false;
                if (style.getPropertyValue("font-family") !== style.fontFamily) return false;
            }
            return true;
        })()"#,
    );
}

#[test]
fn exposes_mouse_keyboard_events_and_document_hit_testing() {
    let page = page_with(
        "<html><head><style>html,body{margin:0}#box{width:100px;height:100px}</style></head>\
         <body><div id='box'></div></body></html>",
    );

    assert_js(
        &page,
        r#"(() => {
            const event = new MouseEvent("mousedown", {
                bubbles: true, clientX: 12, clientY: 34, button: 1, buttons: 2,
                ctrlKey: true, modifierAltGraph: true,
            });
            return event instanceof UIEvent && event instanceof Event && event.bubbles &&
                event.x === 12 && event.y === 34 && event.pageX === 12 && event.button === 1 &&
                event.buttons === 2 && event.getModifierState("Control") &&
                event.getModifierState("AltGraph") && !event.getModifierState("Shift");
        })()"#,
    );
    assert_js(
        &page,
        "new KeyboardEvent('keydown', { shiftKey: true }).getModifierState('Shift')",
    );
    assert_js(
        &page,
        "document.elementFromPoint(10, 10) === document.getElementById('box')",
    );
    assert_js(&page, "document.elementFromPoint(-1, 10) === null");
    assert_js(
        &page,
        "(() => { try { document.elementFromPoint(NaN, 0); return false; } catch (error) { return error instanceof TypeError; } })()",
    );
    assert_js(
        &page,
        "document.body.children.length === 1 && document.body.firstElementChild.id === 'box' && document.body.lastElementChild === document.body.firstElementChild",
    );
}

#[test]
fn exposes_tag_specific_embedded_element_reflections() {
    let page = page_with(
        "<html><body><img id='image' alt='sample' width='42'>\
         <iframe id='frame' width='80%'></iframe><video id='video' controls></video>\
         <canvas id='canvas'></canvas><map><area id='area' href='/next'></map></body></html>",
    );

    assert_js(
        &page,
        r#"(() => {
            const image = document.getElementById("image");
            const frame = document.getElementById("frame");
            const video = document.getElementById("video");
            const canvas = document.getElementById("canvas");
            const area = document.getElementById("area");
            if (!(image instanceof HTMLImageElement) || image.alt !== "sample" || image.width !== 42) return false;
            image.border = null;
            if (image.getAttribute("border") !== "") return false;
            image.border = undefined;
            if (image.getAttribute("border") !== "undefined") return false;
            image.isMap = true;
            if (!image.hasAttribute("ismap")) return false;
            if (!(frame instanceof HTMLIFrameElement) || frame.width !== "80%") return false;
            if (!(video instanceof HTMLVideoElement) || !(video instanceof HTMLMediaElement) || !video.controls) return false;
            if (!(canvas instanceof HTMLCanvasElement) || canvas.width !== 300 || canvas.height !== 150) return false;
            return area instanceof HTMLAreaElement && area.href.endsWith("/next");
        })()"#,
    );
}

#[test]
fn exposes_form_table_and_metadata_element_reflections() {
    let page = page_with(
        "<html><head><link id='link' rel='stylesheet'></head><body>\
         <form id='form' method='POST'><input id='input' required size='12'></form>\
         <table id='table'><tbody><tr><td id='cell' colspan='4'></td></tr></tbody></table>\
         </body></html>",
    );

    assert_js(
        &page,
        r#"(() => {
            const form = document.getElementById("form");
            const input = document.getElementById("input");
            const table = document.getElementById("table");
            const cell = document.getElementById("cell");
            const link = document.getElementById("link");
            if (!(form instanceof HTMLFormElement) || form.method !== "post") return false;
            if (!(input instanceof HTMLInputElement) || !input.required || input.size !== 12) return false;
            input.maxLength = 9;
            if (input.getAttribute("maxlength") !== "9") return false;
            if (!(table instanceof HTMLTableElement) || !(cell instanceof HTMLTableCellElement) || cell.colSpan !== 4) return false;
            return link instanceof HTMLLinkElement && link.relList.contains("stylesheet");
        })()"#,
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
    assert_js(
        &page,
        "(() => { const link = document.getElementById('link'); link.search = '?next=2'; link.hash = 'section'; return link.search === '?next=2' && link.hash === '#section' && String(link) === 'https://example.test/item?next=2#section'; })()",
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
fn validates_and_tracks_custom_element_definitions() {
    let page = page_with("<body></body>");
    assert_js(
        &page,
        r#"(() => {
            function ExampleElement() {}
            customElements.define("example-element", ExampleElement);
            if (customElements.get("example-element") !== ExampleElement ||
                customElements.getName(ExampleElement) !== "example-element") return false;
            let resolved = false;
            customElements.whenDefined("example-element").then(value => {
                resolved = value === ExampleElement;
            });
            try { customElements.define("Invalid-Element", function() {}); return false; }
            catch (error) { return error instanceof DOMException && error.name === "SyntaxError"; }
        })()"#,
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
fn selectors_scan_wide_subtrees_without_quadratic_sibling_traversal() {
    let children = (0..5_000)
        .map(|index| {
            let class = if index % 1_000 == 0 { "hit" } else { "miss" };
            format!("<span class='{class}' data-index='{index}'></span>")
        })
        .collect::<String>();
    let page = page_with(&format!(
        "<html><body><main>{children}</main></body></html>"
    ));

    assert_js(
        &page,
        r#"(() => {
            const main = document.querySelector("main");
            const hits = main.querySelectorAll(":scope > span.hit[data-index]");
            return hits.length === 5 && hits[0].getAttribute("data-index") === "0" &&
                hits[4].getAttribute("data-index") === "4000" &&
                Array.from(main.children).every(element =>
                    element.matches("span.hit") === element.classList.contains("hit"));
        })()"#,
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
            if (two.previousSibling !== null || two.nextSibling !== null) throw new Error("sibling traversal was stale");
            if (!body.contains(two) || !two.isConnected || two.ownerDocument !== document) throw new Error("node relationships were incorrect");
            const three = document.createElement("div");
            body.appendChild(three);
            if (two.compareDocumentPosition(three) !== Node.DOCUMENT_POSITION_FOLLOWING) throw new Error("following order was incorrect");
            if (body.compareDocumentPosition(three) !== (Node.DOCUMENT_POSITION_FOLLOWING | Node.DOCUMENT_POSITION_CONTAINED_BY)) throw new Error("containment order was incorrect");
            const clone = two.cloneNode(true);
            if (clone === two || clone.id !== two.id || clone.parentNode !== null || clone.isConnected) throw new Error("clone was incorrect");
        })()
        "#,
    )
    .unwrap();
}

#[test]
fn parent_and_child_convenience_methods_mutate_the_native_tree() {
    let page = page_with("<main id='root'><i id='old'>old</i></main>");

    assert_js(
        &page,
        r##"(() => {
            const root = document.getElementById("root");
            const old = document.getElementById("old");
            const strong = document.createElement("strong");
            strong.textContent = "strong";
            root.prepend("before", strong);
            root.append("after");
            const correct = root.hasChildNodes() && root.textContent === "beforestrongoldafter";
            old.remove();
            return correct && old.parentNode === null && root.textContent === "beforestrongafter";
        })()"##,
    );

    assert_js(
        &page,
        r#"(() => {
            const root = document.getElementById("root");
            root.replaceChildren("replacement");
            return root.childNodes.length === 1 &&
                root.firstChild.nodeType === 3 && root.textContent === "replacement";
        })()"#,
    );
    assert_js(
        &page,
        "(() => { const text = new Text('abcd'); text.replaceData(1, 2, 'X'); text.appendData('!'); return text.data === 'aXd!' && text.substringData(1, 2) === 'Xd' && text.nodeValue === 'aXd!'; })()",
    );
}

#[test]
fn exposes_ranges_and_the_document_selection() {
    let page = page_with("<main><span>one</span><span>two</span></main>");

    assert_js(
        &page,
        r#"(() => {
            const main = document.querySelector("main");
            const firstText = main.firstChild.firstChild;
            const lastText = main.lastChild.firstChild;
            const range = document.createRange();
            range.setStart(firstText, 1);
            range.setEnd(lastText, 2);
            if (range.collapsed || range.commonAncestorContainer !== main) return false;
            const selection = getSelection();
            selection.removeAllRanges();
            selection.addRange(range);
            if (selection.rangeCount !== 1 || selection.getRangeAt(0) !== range) return false;
            selection.collapse(lastText, 1);
            selection.extend(firstText, 2);
            return selection.anchorNode === lastText && selection.anchorOffset === 1 &&
                selection.focusNode === firstText && selection.focusOffset === 2 &&
                selection.getRangeAt(0).startContainer === firstText &&
                document.getSelection() === selection && selection.type === "Range";
        })()"#,
    );
}

#[test]
fn selectors_search_detached_element_subtrees() {
    let page = page_with("<main><div contenteditable><span id='inside'></span></div></main>");

    assert_js(
        &page,
        r##"(() => {
            const clone = document.documentElement.cloneNode(true);
            return clone.querySelector("[contenteditable]") !== null &&
                clone.querySelector("#inside").id === "inside" &&
                clone.querySelectorAll("main span").length === 1;
        })()"##,
    );
}

#[test]
fn creates_namespaced_elements_and_inserts_adjacent_html() {
    let page = page_with("<main><p id='target'>middle</p></main>");

    assert_js(
        &page,
        r#"(() => {
            const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
            if (svg.localName !== "svg" || svg.namespaceURI !== "http://www.w3.org/2000/svg") return false;
            const qualified = document.createElementNS("https://example.test/ns", "pickle:dill");
            if (qualified.localName !== "dill" || qualified.prefix !== "pickle" ||
                qualified.namespaceURI !== "https://example.test/ns") return false;
            const target = document.getElementById("target");
            target.insertAdjacentHTML("beforebegin", "<b>before</b>");
            target.insertAdjacentHTML("afterbegin", "<i>first</i>");
            target.insertAdjacentHTML("beforeend", "<i>last</i>");
            target.insertAdjacentHTML("afterend", "<b>after</b>");
            return document.querySelector("main").textContent === "beforefirstmiddlelastafter";
        })()"#,
    );
}

#[test]
fn creates_comments_and_splices_document_fragments() {
    let page = page_with("<main>A<!--parsed-->B</main>");
    assert_js(
        &page,
        r##"(() => {
            const main = document.querySelector("main");
            const parsed = main.childNodes[1];
            if (!(parsed instanceof Comment) || !(parsed instanceof CharacterData) ||
                parsed.nodeType !== Node.COMMENT_NODE || parsed.nodeName !== "#comment" ||
                parsed.data !== "parsed" || main.textContent !== "AB") return false;
            parsed.appendData("!");
            if (parsed.nodeValue !== "parsed!") return false;

            const fragment = new DocumentFragment();
            const first = document.createElement("i");
            first.id = "first";
            fragment.append(first, new Comment("between"), "tail");
            const clone = fragment.cloneNode(true);
            if (fragment.nodeType !== Node.DOCUMENT_FRAGMENT_NODE ||
                fragment.nodeName !== "#document-fragment" ||
                fragment.getElementById("first") !== first ||
                !(clone.childNodes[1] instanceof Comment) ||
                clone.childNodes[1].data !== "between") return false;
            main.appendChild(fragment);
            return fragment.childNodes.length === 0 && main.lastChild.data === "tail" &&
                main.querySelector("#first") === first;
        })()"##,
    );
}

#[test]
fn parses_detached_html_and_xml_documents_in_the_canonical_dom() {
    let page = page_with(
        "<html><head><title>Creator</title></head><body><p id='main'>main</p></body></html>",
    );

    assert_js(
        &page,
        r##"(() => {
            const parsed = new DOMParser().parseFromString(
                "<!doctype html><title>Parsed</title><main id='main'><!--note--><p name='item'>text</p></main>",
                "text/html"
            );
            if (!(parsed instanceof Document) || parsed === document ||
                parsed.ownerDocument !== null || parsed.URL !== document.URL ||
                parsed.contentType !== "text/html" || parsed.title !== "Parsed" ||
                parsed.baseURI !== document.URL || parsed.location !== null ||
                parsed.compatMode !== "CSS1Compat" ||
                parsed.documentElement.localName !== "html" ||
                parsed.getElementById("main").localName !== "main" ||
                parsed.getElementById("main") === document.getElementById("main") ||
                parsed.getElementsByName("item").length !== 1 ||
                parsed.querySelector("main").childNodes[0].data !== "note") return false;

            const created = parsed.createElement("aside");
            const fragment = parsed.createDocumentFragment();
            fragment.appendChild(created);
            if (created.ownerDocument !== parsed || fragment.ownerDocument !== parsed) return false;
            document.body.appendChild(created);
            if (created.ownerDocument !== document) return false;

            const xml = new DOMParser().parseFromString(
                "<svg xmlns='http://www.w3.org/2000/svg'><g id='group'>ok</g></svg>",
                "image/svg+xml"
            );
            if (xml instanceof XMLDocument || xml.contentType !== "image/svg+xml" ||
                xml.baseURI !== document.URL ||
                xml.documentElement.namespaceURI !== "http://www.w3.org/2000/svg" ||
                xml.getElementById("group").ownerDocument !== xml) return false;

            const malformed = new DOMParser().parseFromString("<one><two></one>", "application/xml");
            return malformed.documentElement.localName === "parsererror";
        })()"##,
    );

    assert_js(
        &page,
        "(() => { try { new DOMParser().parseFromString('<p>x</p>', 'text/plain'); return false; } catch (error) { return error instanceof TypeError; } })()",
    );
    assert_js(
        &page,
        "new DOMParser().parseFromString('<body><noscript><p>shown</p></noscript>', 'text/html').body.firstChild.firstChild instanceof HTMLParagraphElement",
    );
    assert_js(
        &page,
        "(() => { const parsed = new DOMParser().parseFromString('<template><i id=inside></i></template>', 'text/html'); const content = parsed.querySelector('template').content; return content instanceof DocumentFragment && content.getElementById('inside') !== null && content.getElementById('') === null; })()",
    );
}

#[test]
fn exposes_text_encoding_api() {
    let mut page = page_with("<title>Encoding</title>");

    assert_js(
        &page,
        "new TextDecoder('latin1').encoding === 'windows-1252'",
    );
    assert_js(
        &page,
        "(() => { try { new TextDecoder('\\u00A0utf-8'); return false; } catch (error) { return error instanceof RangeError; } })()",
    );
    assert_js(
        &page,
        "new TextDecoder('windows-1252').decode(Uint8Array.of(0x80)) === '€'",
    );
    assert_js(
        &page,
        "new TextDecoder().decode(Uint8Array.of(0xEF, 0xBB, 0xBF, 65)) === 'A'",
    );
    assert_js(
        &page,
        "new TextDecoder('utf-8', { ignoreBOM: true }).decode(Uint8Array.of(0xEF, 0xBB, 0xBF, 65)) === '\u{FEFF}A'",
    );
    assert_js(
        &page,
        "new TextDecoder().decode(Uint8Array.of(0)).charCodeAt(0) === 0",
    );
    assert_js(
        &page,
        "(() => { try { new TextDecoder('utf-8', { fatal: true }).decode(Uint8Array.of(0xFF)); return false; } catch (error) { return error instanceof TypeError; } })()",
    );
    assert_js(
        &page,
        "[...new TextEncoder().encode('A🦀')].join(',') === '65,240,159,166,128'",
    );
    assert_js(
        &page,
        "[...new TextEncoder().encode('A\\uD800Z')].join(',') === '65,239,191,189,90'",
    );
    assert_js(
        &page,
        "(() => { const output = new Uint8Array(5); const result = new TextEncoder().encodeInto('A🦀Z', output); return result.read === 3 && result.written === 5 && output[4] === 128; })()",
    );
    assert_js(
        &page,
        "(() => { const decoder = new TextDecoder(); return decoder.decode(Uint8Array.of(0xF0, 0x9F), { stream: true }) === '' && decoder.decode(Uint8Array.of(0xA6, 0x80), { stream: true }) === '🦀' && decoder.decode() === ''; })()",
    );
    assert_js(&page, "document.characterSet === 'UTF-8'");
    assert_js(
        &page,
        "crossOriginIsolated === false && typeof SharedArrayBuffer === 'undefined' && new WebAssembly.Memory({ shared: true, initial: 0, maximum: 0 }).buffer.constructor.name === 'SharedArrayBuffer'",
    );
    assert_js(
        &page,
        "(() => { const decoder = new TextDecoder('iso-2022-jp', { fatal: true }); try { decoder.decode(Uint8Array.of(0x1b, 0x28, 0x4a, 0xff), { stream: true }); } catch (error) { return error instanceof TypeError && decoder.decode(Uint8Array.of(0x7e)) === '‾'; } return false; })()",
    );
    assert_js(
        &page,
        "(() => { const buffer = new ArrayBuffer(10); const view = new Uint8Array(buffer); new MessageChannel().port1.postMessage(buffer, [buffer]); const result = new TextEncoder().encodeInto('test', view); return buffer.byteLength === 0 && view.byteLength === 0 && result.read === 0 && result.written === 0; })()",
    );
    assert_js(
        &page,
        "(() => { const buffer = new ArrayBuffer(8); const view = new Uint8Array(buffer); view.fill(42); const options = { get stream() { new MessageChannel().port1.postMessage(buffer, [buffer]); return false; } }; return new TextDecoder().decode(view, options) === ''; })()",
    );
    assert_js(
        &page,
        "(() => { const channel = new MessageChannel(); channel.port2.onmessage = event => { globalThis.__messageData = event.data; globalThis.__trustedMessage = event.isTrusted; }; channel.port1.postMessage('hello'); return true; })()",
    );
    page.run_pending_tasks().unwrap();
    assert_js(
        &page,
        "globalThis.__messageData === 'hello' && globalThis.__trustedMessage === true",
    );
}

#[test]
fn serializes_legacy_encoded_anchor_queries() {
    let big5 = page_with("<meta charset='big5'>");
    assert_js(
        &big5,
        "(() => { const anchor = document.createElement('a'); anchor.href = 'https://example.com/?X　X'; return anchor.search === '?X%A1@X'; })()",
    );
    assert_js(
        &big5,
        "(() => { const anchor = document.createElement('a'); anchor.href = 'https://example.com/?☃'; return anchor.search === '?%26%239731%3B'; })()",
    );

    let iso_2022_jp = page_with("<meta charset='iso-2022-jp'>");
    assert_js(
        &iso_2022_jp,
        "(() => { const anchor = document.createElement('a'); anchor.href = 'https://example.com/?\\u00A5\\u203Es\\\\\\uFF90\\u4F69'; return anchor.search === '?%1B(J\\\\~s%1B(B\\\\%1B$B%_PP%1B(B'; })()",
    );
}

#[test]
fn submits_legacy_encoded_get_forms_to_named_iframes() {
    let mut page = page_with(
        "<meta charset='big5'><iframe id='result' name='result'></iframe><form id='form' method='get' action='https://example.com/common/blank.html' accept-charset='big5' target='result'><input name='value' value='initial'></form>",
    );
    assert_js(
        &page,
        r#"(() => {
            const input = document.querySelector("input");
            const form = document.querySelector("form");
            const frame = document.querySelector("iframe");
            input.value = "X　☃X";
            form.submit();
            frame.onload = () => { globalThis.__frameLoaded = true; };
            return input.defaultValue === "initial" && input.value === "X　☃X" &&
                frame.contentWindow.location.pathname === "/common/blank.html" &&
                frame.contentWindow.location.search === "?value=X%A1%40%26%239731%3BX";
        })()"#,
    );
    page.run_pending_tasks().unwrap();
    assert_js(&page, "globalThis.__frameLoaded === true");
}

#[test]
fn executes_data_iframes_and_delivers_window_messages() {
    let mut page = page_with("<meta charset='utf-8'><body></body>");
    assert_js(
        &page,
        r#"(() => {
            const frame = document.createElement("iframe");
            frame.name = "target";
            document.body.appendChild(frame);
            const childComment = new frame.contentWindow.Comment("child");
            if (frame.contentDocument !== childComment.ownerDocument) return false;

            addEventListener("message", event => { globalThis.__iframeMessage = event.data; });
            const form = document.createElement("form");
            form.acceptCharset = "iso-2022-jp";
            form.target = "target";
            form.action = "data:text/html;charset=iso-2022-jp," + escape(
                '<body onload="parent.postMessage({text: document.body.innerText.split(\'=\').pop(), raw: unescape(location.href.split(\'=\').pop())}, \'*\')"><plaintext>'
            );
            const input = document.createElement("input");
            input.name = "value";
            input.value = "ABC★星🌟";
            form.appendChild(input);
            document.body.appendChild(form);
            form.submit();
            return frame.contentWindow.location.protocol === "data:";
        })()"#,
    );
    page.run_pending_tasks().unwrap();
    let message = page
        .eval("JSON.stringify(globalThis.__iframeMessage)")
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(
        message,
        r#"{"text":"ABC★星&#127775;","raw":"ABC\u001b$B!z@1\u001b(B&#127775;"}"#,
    );
}

#[test]
fn exposes_base64_api() {
    let page = page_with("<title>Base64</title>");

    assert_js(&page, "btoa('hello\\0ÿ') === 'aGVsbG8A/w=='");
    assert_js(&page, "atob(' aGVs\\nbG8A/w ') === 'hello\\0ÿ'");
    assert_js(
        &page,
        "(() => { try { btoa('€'); return false; } catch (error) { return error instanceof DOMException && error.name === 'InvalidCharacterError'; } })()",
    );
    assert_js(
        &page,
        "(() => { try { atob('!'); return false; } catch (error) { return error instanceof DOMException && error.name === 'InvalidCharacterError'; } })()",
    );
}

#[test]
fn exposes_blob_api() {
    let page = page_with("<title>Blob</title>");

    assert_js(
        &page,
        "(() => { const blob = new Blob(['A', Uint8Array.of(0, 255)], { type: 'Text/Plain' }); return blob.size === 3 && blob.type === 'text/plain' && blob.slice(1, 3, 'X/Test').size === 2 && blob.slice(1).type === ''; })()",
    );
    assert_js(
        &page,
        "self.Blob === Blob && self.Headers === Headers && self.Response === Response",
    );
    assert_js(
        &page,
        "(() => { let value = false; new Blob(['hello']).text().then(text => value = text === 'hello'); queueMicrotask(() => {}); return true; })()",
    );
    assert_js(
        &page,
        "(() => { const response = new Response('body', { headers: { 'Content-Type': 'text/plain' } }); let value = false; response.blob().then(blob => value = blob.size === 4 && blob.type === 'text/plain'); return true; })()",
    );
    assert_js(
        &page,
        r#"(() => {
            const cases = [
                ["TEXT/HTML;CHARSET=GBK", "text/html;charset=GBK"],
                ["text/html;charset=gbk(", "text/html;charset=\"gbk(\""],
                ["text/html;charset=gbk;charset=windows-1255", "text/html;charset=gbk"],
                ["text/html;charset =gbk", "text/html"],
                ["not a mime type", ""],
            ];
            return cases.every(([input, output]) =>
                new Blob([], { type: input }).type === output &&
                new File([], "file", { type: input }).type === output
            );
        })()"#,
    );
    assert_js(
        &page,
        r#"(() => {
            const headers = new Headers({ "Content-Type": "  TEXT/HTML;CHARSET=GBK\t" });
            if (headers.get("content-type") !== "TEXT/HTML;CHARSET=GBK") return false;
            if (new Headers(headers).get("content-type") !== "TEXT/HTML;CHARSET=GBK") return false;
            for (const value of ["text/html\0", "text/html\nfoo", "text/星"]) {
                try { new Headers([["Content-Type", value]]); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
            }
            return true;
        })()"#,
    );
}

#[test]
fn exposes_form_data_api_and_constructs_successful_form_controls() {
    let page = page_with(
        "<form id='form'>\
           <input name='title' value='hello'>\
           <input name='enabled' type='checkbox' checked>\
           <input name='ignored' type='checkbox'>\
           <select name='choice'><option value='a'>A</option><option value='b' selected>B</option></select>\
           <textarea name='notes'>line one</textarea>\
           <button id='submit' name='action' value='save'>Save</button>\
         </form>\
         <input name='external' value='yes' form='form'>",
    );

    let entries = page
        .eval(
            r#"(() => {
            const form = document.getElementById("form");
            const submit = document.getElementById("submit");
            form.addEventListener("formdata", event => event.formData.append("event", "seen"));
            const data = new FormData(form, submit);
            globalThis.__testedFormData = data;
            return JSON.stringify([...data]);
        })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(
        entries,
        r#"[["title","hello"],["enabled","on"],["choice","b"],["notes","line one"],["action","save"],["external","yes"],["event","seen"]]"#
    );
    assert_js(
        &page,
        "Object.prototype.toString.call(__testedFormData) === '[object FormData]' && Object.getOwnPropertyDescriptor(FormData.prototype, 'append').enumerable",
    );
    assert_js(
        &page,
        r#"(() => {
            const data = new FormData();
            data.append("name", "first");
            data.append("name", "second");
            data.append("file", new Blob([Uint8Array.of(0, 255)], { type: "application/x-test" }), "data.bin");
            if (data.get("name") !== "first" || data.getAll("name").join(",") !== "first,second") return false;
            const file = data.get("file");
            const request = new Request("https://example.test/upload", { method: "POST", body: data });
            const clone = request.clone();
            const binary = new Request("https://example.test/binary", { method: "POST", body: Uint8Array.of(0, 255) });
            data.set("name", "replacement");
            data.delete("file");
            return file instanceof File && file.name === "data.bin" && file.size === 2 &&
                file.type === "application/x-test" && data.getAll("name").join(",") === "replacement" &&
                !data.has("file") && [...data.keys()].join(",") === "name" &&
                clone.headers.get("content-type") === request.headers.get("content-type") &&
                clone.__bodyBytes !== request.__bodyBytes &&
                clone.__bodyBytes.length === request.__bodyBytes.length &&
                binary.__bodyBytes[0] === 0 && binary.__bodyBytes[1] === 255;
        })()"#,
    );
    assert_js(
        &page,
        "(() => { try { FormData.prototype.append.call({}, 'x', 'y'); return false; } catch (error) { return error instanceof TypeError; } })()",
    );
}

#[test]
fn exposes_file_api_and_validates_blob_inputs() {
    let page = page_with("<title>Files</title>");

    assert_js(
        &page,
        r#"(() => {
            const file = new File(["ab", Uint8Array.of(99)], "a\uD800b.txt", {
                type: "TEXT/PLAIN", lastModified: 42,
            });
            if (!(file instanceof Blob) || file.size !== 3 || file.name !== "a\uFFFDb.txt" ||
                file.type !== "text/plain" || file.lastModified !== 42) return false;
            for (const value of [null, true, 1, "bad", {}]) {
                try { new Blob(value); return false; } catch (error) {
                    if (!(error instanceof TypeError)) return false;
                }
            }
            try { new Blob([], { endings: "invalid" }); return false; } catch (error) {
                return error instanceof TypeError;
            }
        })()"#,
    );
}

#[test]
fn exposes_web_storage_and_storage_events() {
    let page = page_with("<title>Storage</title>");

    assert_js(
        &page,
        r#"(() => {
            localStorage.clear();
            localStorage.setItem("name", "first");
            localStorage.name = "second";
            localStorage["\uD800"] = "\uDC00";
            if (localStorage.length !== 2 || localStorage.name !== "second" ||
                localStorage.getItem("\uD800") !== "\uDC00") return false;
            if (!Object.keys(localStorage).includes("name") || !("name" in localStorage)) return false;
            delete localStorage.name;
            return localStorage.getItem("name") === null && localStorage.length === 1;
        })()"#,
    );
    assert_js(
        &page,
        r#"(() => {
            sessionStorage.clear();
            Storage.prototype.shadowed = "prototype";
            sessionStorage.shadowed = "stored";
            const result = sessionStorage.shadowed === "prototype" &&
                sessionStorage.getItem("shadowed") === "stored" &&
                Object.getOwnPropertyDescriptor(sessionStorage, "shadowed") === undefined;
            delete Storage.prototype.shadowed;
            sessionStorage.clear();
            return result;
        })()"#,
    );
    assert_js(
        &page,
        r#"(() => {
            const event = new StorageEvent("storage", {
                bubbles: true, key: "key", oldValue: "old", newValue: "new",
                url: "relative", storageArea: localStorage,
            });
            event.initStorageEvent("changed", false, true, null, null, "value", "url", sessionStorage);
            return event instanceof Event && event.type === "changed" && !event.bubbles &&
                event.cancelable && event.key === null && event.newValue === "value" &&
                event.url === "url" && event.storageArea === sessionStorage;
        })()"#,
    );
}

#[test]
fn exposes_request_api() {
    let page = page_with("<title>Request</title>");

    assert_js(
        &page,
        "(() => { const request = new Request('https://example.test/api', { method: 'post', headers: { Accept: 'text/plain' }, body: 'data' }); const clone = request.clone(); return request.url === 'https://example.test/api' && request.method === 'POST' && request.headers.get('accept') === 'text/plain' && clone !== request && clone.signal !== request.signal; })()",
    );
    assert_js(
        &page,
        "(() => { try { new Request('/api', { method: 'GET', body: 'no' }); return false; } catch (error) { return error instanceof TypeError; } })()",
    );
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
