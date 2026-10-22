use brimp_runtime::runtime::{Page, PageOptions, Session};

fn page() -> Page {
    let browser = Session::default();
    let mut page = browser.new_local_page(PageOptions::default()).unwrap();
    page.set_content("<!doctype html><main id='main'><b>before</b></main>")
        .unwrap();
    page
}
fn check(page: &Page, source: &str) {
    let value = page
        .eval(&format!("JSON.stringify(({source}))"))
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(value, "true", "{source}");
}
#[test]
fn replacements_detach_nodes_without_invalidating_wrappers() {
    let page = page();
    check(
        &page,
        r##"(() => {
        const main = document.querySelector('main'), original = main.firstChild;
        const text = original.firstChild, style = original.style;
        main.innerHTML = '<i>next</i>';
        if (original.parentNode !== null || original.textContent !== 'before') return false;
        original.textContent = 'changed';
        if (text.parentNode !== null || text.data !== 'before') return false;
        style.color = 'red';
        main.appendChild(original);
        return original === main.lastChild && original.style === style && original.style.color === 'red';
    })()"##,
    );
}
#[test]
fn detached_documents_and_templates_keep_canonical_ownership() {
    let page = page();
    check(
        &page,
        r##"(() => {
        const parsed = new DOMParser().parseFromString('<main><template><b>inside</b></template><p>other</p></main>', 'text/html');
        const template = parsed.querySelector('template');
        const content = template.content;
        if (content !== template.content || content.ownerDocument !== parsed || content.firstChild.ownerDocument !== parsed) return false;
        if (template.childNodes.length !== 0 || parsed.querySelector('b') !== null) return false;
        const child = parsed.querySelector('p'); child.remove();
        if (child.ownerDocument !== parsed) return false;
        document.body.appendChild(child);
        return child.ownerDocument === document && child.textContent === 'other' && template.innerHTML === '<b>inside</b>';
    })()"##,
    );
}
#[test]
fn template_cloning_copies_inert_content() {
    let page = page();
    check(
        &page,
        r##"(() => {
        const template = document.createElement('template');
        template.innerHTML = '<p>first<template><b>nested</b></template></p>';
        const copy = template.cloneNode(true);
        return copy !== template && copy.content.firstChild !== template.content.firstChild &&
            copy.innerHTML === template.innerHTML && copy.content.querySelector('template').content.textContent === 'nested';
    })()"##,
    );
}
#[test]
fn lexbor_selectors_declarations_and_namespaced_attributes() {
    let page = page();
    check(
        &page,
        r##"(() => {
        const main = document.querySelector('main');
        if (main.querySelector(':scope > b') !== main.firstChild || !main.matches(':has(> b)')) return false;
        main.innerHTML = '<svg viewBox="0 0 1 1" xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="#a"/></svg>';
        const use = main.querySelector('use'), attr = use.attributes.getNamedItem('xlink:href');
        if (attr.namespaceURI !== 'http://www.w3.org/1999/xlink' || attr.prefix !== 'xlink' || attr.localName !== 'href') return false;
        const style = main.style; style.cssText = 'color: red; color: blue !important; color: green; --message: "a;b"';
        if (style.color !== 'blue' || style.getPropertyPriority('color') !== 'important') return false;
        style.setProperty('padding', '2px');
        return style.padding === '2px' && style.getPropertyValue('--message').includes('a;b');
    })()"##,
    );
}
#[test]
fn core_has_no_layout_or_xml_parser() {
    let page = page();
    check(
        &page,
        r##"(() => {
        if ('getBoundingClientRect' in Element.prototype || 'clientWidth' in Element.prototype ||
            'getComputedStyle' in globalThis || 'elementFromPoint' in Document.prototype) return false;
        try { new DOMParser().parseFromString('<a/>', 'application/xml'); return false; }
        catch (error) { return error instanceof TypeError; }
    })()"##,
    );
}

#[test]
fn mutation_records_cover_attributes_text_removal_and_microtasks() {
    let mut page = page();
    page.eval(
        r##"
        globalThis.records = [];
        const main = document.querySelector('main');
        const observer = new MutationObserver(batch => records.push(...batch));
        observer.observe(main, {subtree: true, attributes: true, attributeOldValue: true,
            characterData: true, characterDataOldValue: true, childList: true});
        main.setAttribute('title', 'one');
        main.setAttribute('title', 'two');
        main.firstChild.firstChild.data = 'changed';
        const removed = main.firstChild;
        removed.remove();
        removed.setAttribute('data-detached', 'still observed until delivery');
        globalThis.synchronousCount = records.length;
    "##,
    )
    .unwrap();
    page.run_pending_tasks().unwrap();
    check(
        &page,
        r##"synchronousCount === 0 && records.length === 5 &&
        records[0].type === 'attributes' && records[0].attributeName === 'title' && records[0].oldValue === null &&
        records[1].oldValue === 'one' && records[2].type === 'characterData' && records[2].oldValue === 'before' &&
        records[3].removedNodes[0] === removed && records[4].attributeName === 'data-detached'"##,
    );
    page.eval("observer.disconnect(); removed.setAttribute('later', 'unobserved')")
        .unwrap();
    page.run_pending_tasks().unwrap();
    check(
        &page,
        "records.length === 5 && observer.takeRecords().length === 0",
    );
}

#[test]
fn input_handler_detection_bubbling_and_rel_capabilities() {
    check(
        &page(),
        r#"(() => {
        const input = document.createElement('input'); document.body.append(input);
        let seen = '';
        document.oninput = e => { if (e.target === input) seen += 'document'; };
        input.oninput = () => { seen += 'input,'; };
        input.dispatchEvent(new InputEvent('input', {bubbles:true}));
        input.oninput = 123;
        let classError = false;
        try { input.classList.supports('x'); } catch(e) { classError = e instanceof TypeError; }
        return 'oninput' in document && input.oninput === null && seen === 'input,document' &&
            document.createElement('link').relList.supports('modulepreload') === false && classError;
    })()"#,
    );
}

#[test]
fn node_iterator_visits_root_skips_filters_and_tracks_removal() {
    check(
        &page(),
        r#"(() => {
        const root = document.createElement('div'); root.innerHTML = '<b><i>A</i></b><p>B</p>';
        const iterator = document.createNodeIterator(root, NodeFilter.SHOW_ELEMENT, n => n.localName === 'b' ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT);
        if (iterator.nextNode() !== root || iterator.nextNode().localName !== 'i') return false;
        root.firstChild.remove();
        if (iterator.referenceNode !== root || iterator.nextNode().localName !== 'p') return false;
        return iterator.previousNode().localName === 'p' && iterator.previousNode() === root;
    })()"#,
    );
}

#[test]
fn clone_preserves_cycles_maps_views_and_transfer() {
    check(
        &page(),
        r#"(() => {
        const buffer = new ArrayBuffer(8), view = new Uint8Array(buffer); view[2] = 42;
        const value = {buffer, view, map: new Map(), date:new Date(123)}; value.self = value; value.map.set(value, view);
        const copy = structuredClone(value, {transfer: [buffer]});
        let rejected = false;
        try { structuredClone(() => 1); } catch (e) { rejected = e.name === 'DataCloneError'; }
        return rejected && buffer.byteLength === 0 && copy.self === copy && copy.buffer === copy.view.buffer &&
            copy.map.get(copy) === copy.view && copy.view[2] === 42 && +copy.date === 123;
    })()"#,
    );
}

#[test]
fn xpath_uses_live_dom_and_namespace_resolvers() {
    check(
        &page(),
        r#"(() => {
        const root = document.createElement('div'); root.innerHTML = '<p n="1">A</p><p n="2">B</p>';
        const evaluator = new XPathEvaluator();
        const expression = evaluator.createExpression('.//p[@n="2"]');
        const result = expression.evaluate(root, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
        root.lastChild.textContent = 'changed';
        return result.singleNodeValue === root.lastChild && document.evaluate('string(.//p[@n="2"])', root, null, XPathResult.STRING_TYPE, null).stringValue === 'changed';
    })()"#,
    );
}

#[test]
fn shadow_trees_slots_and_composed_events_preserve_boundaries() {
    check(
        &page(),
        r#"(() => {
        const host = document.createElement('div'), light = document.createElement('b');
        host.append(light); document.body.append(host);
        const shadow = host.attachShadow({mode:'closed'});
        shadow.innerHTML = '<slot></slot><button>inside</button>';
        const slot = shadow.firstChild, button = shadow.lastChild;
        if (host.shadowRoot !== null || light.assignedSlot !== null || slot.assignedNodes()[0] !== light) return false;
        if (document.querySelector('button') !== null || !button.isConnected || button.getRootNode() !== shadow || button.getRootNode({composed:true}) !== document) return false;
        let inside = false, outside = false, escaped = false;
        shadow.addEventListener('test', e => { inside = e.target === button && e.composedPath().includes(button); });
        host.addEventListener('test', e => { outside = e.target === host && !e.composedPath().includes(button); });
        host.addEventListener('private', () => escaped = true);
        button.dispatchEvent(new Event('test', {bubbles:true,composed:true}));
        button.dispatchEvent(new Event('private', {bubbles:true}));
        host.remove();
        return inside && outside && !escaped && !button.isConnected;
    })()"#,
    );
}

#[test]
fn attribute_node_removal_is_live_and_retains_detached_value() {
    check(
        &page(),
        r#"(() => {
        const element = document.createElement('img'); element.setAttribute('onerror','bad()');
        const attr = element.getAttributeNode('onerror');
        if (element.removeAttributeNode(attr) !== attr || element.hasAttribute('onerror')) return false;
        element.setAttribute('onerror','new');
        return attr.ownerElement === null && attr.value === 'bad()' && element.getAttributeNode('onerror') !== attr;
    })()"#,
    );
}

#[test]
fn iterators_recover_from_replacing_all_children() {
    check(
        &page(),
        r#"(() => {
        const root = document.createElement('div'); root.innerHTML = '<b>A</b><i>B</i>';
        const iterator = document.createNodeIterator(root, NodeFilter.SHOW_ELEMENT);
        iterator.nextNode(); iterator.nextNode(); iterator.previousNode();
        root.innerHTML = '<p>C</p>';
        return iterator.referenceNode === root && iterator.nextNode() === root.firstChild;
    })()"#,
    );
}

#[test]
fn custom_element_definitions_finalize_before_construction() {
    check(
        &page(),
        r#"(() => {
        const order = [];
        class Example extends HTMLElement {
            static get observedAttributes() { order.push('attributes'); return ['value']; }
            constructor() { super(); order.push('constructor'); }
            attributeChangedCallback() { order.push('changed'); }
        }
        customElements.define('definition-order', Example);
        const element = document.createElement('definition-order');
        element.setAttribute('value','x');
        return order.join() === 'attributes,constructor,changed';
    })()"#,
    );
}
