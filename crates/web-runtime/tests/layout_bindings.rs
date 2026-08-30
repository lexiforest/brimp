use web_runtime::{Browser, PageOptions};

#[test]
fn style_mutation_restyles_and_relayouts_before_geometry_reads() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(1280, 720).build())
        .unwrap();
    page.set_content(
        r#"
        <html>
        <head>
            <style>
                #box { width: 200px; padding: 20px; border: 5px solid black; }
            </style>
        </head>
        <body><div id="box">Hello</div></body>
        </html>
        "#,
    )
    .unwrap();

    let before = page
        .eval("document.getElementById('box').getBoundingClientRect().width")
        .unwrap()
        .to_number()
        .unwrap();
    assert_eq!(before, 250.0);

    page.eval("document.getElementById('box').style.width = '300px'")
        .unwrap();

    let result = page
        .eval(
            r#"
            (() => {
                const box = document.getElementById("box");
                const rect = box.getBoundingClientRect();
                if (!(rect instanceof DOMRect)) throw new Error("expected DOMRect");
                if (getComputedStyle(box).width !== "300px") throw new Error("bad computed width");
                if (getComputedStyle(box).padding !== "20px") throw new Error("bad padding");
                if (box.clientWidth !== 340) throw new Error("bad clientWidth: " + box.clientWidth);
                if (box.offsetWidth !== 350) throw new Error("bad offsetWidth: " + box.offsetWidth);
                return rect.width;
            })()
            "#,
        )
        .unwrap()
        .to_number()
        .unwrap();
    assert_eq!(result, 350.0);

    let document = page.document();
    let box_id = document.get_element_by_id("box").unwrap();
    assert_eq!(document.bounding_rect(box_id).unwrap()[2], 350.0);
}

#[test]
fn geometry_uses_the_configured_viewport() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(640, 480).build())
        .unwrap();
    page.set_content(
        "<html><head><style>#box { width: 50%; }</style></head><body><div id='box'></div></body></html>",
    )
    .unwrap();

    let width = page
        .eval("document.getElementById('box').getBoundingClientRect().width")
        .unwrap()
        .to_number()
        .unwrap();

    // The body has the default 8px margins, leaving 624px for its content box.
    assert_eq!(width, 312.0);
}

#[test]
fn match_media_exposes_viewport_and_default_preference_queries() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(640, 480).build())
        .unwrap();
    page.set_content("<html><body></body></html>").unwrap();

    let result = page
        .eval(
            r#"(() => {
                const dark = matchMedia("(prefers-color-scheme: dark)");
                const light = matchMedia("(prefers-color-scheme: light)");
                const wide = matchMedia("screen and (min-width: 600px) and (orientation: landscape)");
                const print = matchMedia("print");
                if (!(dark instanceof MediaQueryList) || dark.media !== "(prefers-color-scheme: dark)") return "shape";
                if (dark.matches || !light.matches || !wide.matches || print.matches) return "matches";
                if (typeof dark.addListener !== "function" || typeof dark.removeListener !== "function" || dark.onchange !== null) return "events";
                const event = new MediaQueryListEvent("change", { media: dark.media, matches: true });
                if (!(event instanceof Event) || event.media !== dark.media || !event.matches) return "event";
                try { new MediaQueryList(); return "constructible"; } catch (error) {
                    if (!(error instanceof TypeError)) return "constructor error";
                }
                if (!Function.prototype.toString.call(matchMedia).includes("[native code]")) return "native";
                return "ok";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn computed_style_exposes_stylo_color_serialization() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body><div id='target'></div></body></html>")
        .unwrap();

    let color = page
        .eval(
            "(() => { const target = document.getElementById('target'); target.style.color = 'hsl(120 30% 50% / 0.5)'; return getComputedStyle(target).color; })()",
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(color, "rgba(89, 166, 89, 0.5)");
}

#[test]
fn computed_style_serializes_generic_stylo_longhands() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><body><div id='target' style='display: flex; margin-top: 12px; \
         position: absolute; top: 3px; opacity: .5; line-height: 20px; \
         flex-direction: column'></div></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const style = getComputedStyle(document.getElementById("target"));
                const expected = {
                    display: "flex",
                    marginTop: "12px",
                    position: "absolute",
                    top: "3px",
                    opacity: "0.5",
                    lineHeight: "20px",
                    flexDirection: "column",
                };
                for (const [property, value] of Object.entries(expected)) {
                    if (style[property] !== value) return `${property}: ${style[property]}`;
                }
                for (const property of ["border", "font", "marginTop", "maxWidth", "width"]) {
                    if (!(property in style)) return `supported property: ${property}`;
                }
                const names = [...style];
                for (const property of ["margin-top", "font-size", "max-width", "width"]) {
                    if (!names.includes(property)) return `enumerated property: ${property}`;
                }
                if (document.defaultView !== window || getComputedStyle.name !== "getComputedStyle") {
                    return "window exposure";
                }
                for (const mutation of [
                    () => { style.color = "blue"; },
                    () => { style.setProperty("color", "blue"); },
                    () => { style.removeProperty("color"); },
                ]) {
                    try { mutation(); return "writable computed style"; } catch (error) {
                        if (!(error instanceof DOMException) ||
                            error.name !== "NoModificationAllowedError" || error.code !== 7) {
                            return `wrong readonly error: ${error}`;
                        }
                    }
                }
                return "ok";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn inline_style_exposes_generic_css_idl_properties_through_stylo() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body><div id='target'></div></body></html>")
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const target = document.getElementById("target");
                const first = target.style;
                if (first !== target.style || !(first instanceof CSSStyleDeclaration)) {
                    return "identity";
                }
                target.setAttribute("style", "word-spacing: .1em; background-position: -.5% 0px");
                if (first.wordSpacing !== "0.1em") return "word-spacing getter";
                if (first.backgroundPosition !== "-0.5% 0px") return "background-position getter";
                first.borderTopWidth = ".5px";
                if (first.borderTopWidth !== "0.5px" ||
                    first.getPropertyValue("border-top-width") !== "0.5px") {
                    return "generic setter";
                }
                first.setProperty("color", "red", "important");
                if (first.getPropertyPriority("color") !== "important" ||
                    !first.cssText.includes("color: red !important;") ||
                    first.length !== [...first].length || first.item(0) === "") {
                    return "declaration surface";
                }
                first.cssText = "margin: 1px; color: blue";
                if (first.margin !== "1px" || first.color !== "blue" ||
                    target.getAttribute("style") !== first.cssText) return "cssText";
                return "ok";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn computed_style_exposes_cascaded_custom_properties_from_stylo() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><body style='--inherited: parent'><div id='target'></div></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const target = document.getElementById("target");
                const digits = "1234567890123456789012345";
                target.style.setProperty("--own", digits);
                target.style.zIndex = digits;
                const expectedZIndex = getComputedStyle(target).zIndex;
                target.style.zIndex = "var(--own)";
                if (target.style.getPropertyValue("--own") !== digits) return "inline value";
                const computed = getComputedStyle(target);
                if (computed.getPropertyValue("--inherited") !== "parent") {
                    return "inherited value";
                }
                if (computed.getPropertyValue("--own") !== digits) return "computed value";
                if (computed.zIndex !== expectedZIndex) return `substitution: ${computed.zIndex}`;
                const names = [...computed];
                if (names[names.length - 2] !== "--inherited" ||
                    names[names.length - 1] !== "--own") return `enumeration: ${names}`;
                if (computed[computed.length - 1] !== "--own") return "indexed getter";
                const assigned = document.createElement("span");
                const declaration = assigned.style;
                assigned.style = "--assigned\\;name: value";
                if (assigned.style !== declaration || declaration.length !== 1 ||
                    declaration[0] !== "--assigned;name" ||
                    declaration.getPropertyValue("--assigned;name") !== "value") {
                    return `style forwarding: ${declaration.length}|${declaration[0]}|${declaration.getPropertyValue("--assigned;name")}|${declaration.cssText}`;
                }
                return "ok";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn cssom_rules_are_live_and_mutate_the_stylo_stylesheet() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><head><style>#target { height: 10px; }</style><style></style></head>\
         <body><div id='target'></div></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r##"(() => {
                if (document.styleSheets.length !== 2) return "bad sheet count";
                const sheet = document.styleSheets[1];
                if (sheet !== document.querySelectorAll("style")[1].sheet) return "bad identity";
                if (sheet.cssRules.length !== 0) return "sheet should start empty";
                if (sheet.insertRule("#target { width: 123px; }") !== 0) return "bad index";
                if (!(sheet.cssRules[0] instanceof CSSStyleRule)) return "bad rule type";
                if (sheet.cssRules[0].selectorText !== "#target") return "bad selector";
                if (document.getElementById("target").getBoundingClientRect().width !== 123) {
                    return "rule did not apply";
                }
                sheet.deleteRule(0);
                if (sheet.cssRules.length !== 0) return "rule was not deleted";
                return "ok";
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn css_style_rule_selector_text_is_live_and_preserves_rule_identity() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><head><style>.inactive { color: red; }</style></head>\
         <body><div id='target'></div></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r##"(() => {
                const target = document.getElementById("target");
                const sheet = document.styleSheets[0];
                const rule = sheet.cssRules[0];
                const declaration = rule.style;
                rule.selectorText = "!!";
                if (rule.selectorText !== ".inactive") return "invalid selector changed rule";
                rule.selectorText = "  #target  ";
                if (sheet.cssRules[0] !== rule || rule.selectorText !== "#target") {
                    return "selector or identity";
                }
                declaration.color = "green";
                if (rule.style !== declaration ||
                    getComputedStyle(target).color !== "rgb(0, 128, 0)") {
                    return "declaration or restyle";
                }
                return "ok";
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn keyframe_rules_expose_live_cssom_declarations_and_operations() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><head><style>@media not all { body { color: lime; } } \
         @keyframes slidein { from { margin-left: 100%; } to { margin-left: 0%; } }\
         </style></head><body></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r#"(() => {
                if (Object.prototype.toString.call(CSS) !== "[object CSS]") {
                    return "CSS namespace";
                }
                const media = document.styleSheets[0].cssRules[0];
                if (!(media instanceof CSSConditionRule) || media.conditionText !== "not all") {
                    return "condition rule";
                }
                media.media = "screen and (min-width:480px), print, screen";
                if (media.media.mediaText !== "screen and (min-width: 480px), print, screen" ||
                    media.media[1] !== "print" || !media.cssText.startsWith("@media screen")) {
                    return "media list";
                }
                media.media.deleteMedium("screen");
                if (media.media.mediaText !== "screen and (min-width: 480px), print") {
                    return "delete media";
                }
                media.media.appendMedium("speech, print");
                if (media.media.length !== 2) return "invalid appended media";
                try {
                    media.media.deleteMedium("speech");
                    return "missing medium";
                } catch (error) {
                    if (!(error instanceof DOMException) || error.name !== "NotFoundError") {
                        return "wrong missing-medium error";
                    }
                }
                const keyframes = document.styleSheets[0].cssRules[1];
                if (keyframes.name !== "slidein" || keyframes.length !== 2 ||
                    keyframes[0] !== keyframes.cssRules[0]) return "keyframes surface";
                const from = keyframes.findRule("from");
                if (from.style.marginLeft !== "100%") return "keyframe declaration";
                from.style = "margin-left: 50%; width: 100%;";
                if (from.style.marginLeft !== "50%" || from.style.width !== "100%") {
                    return "keyframe PutForwards";
                }
                keyframes.appendRule("50% { margin-left: 25%; }");
                if (keyframes.length !== 3 || keyframes.findRule("50%").style.marginLeft !== "25%") {
                    return "append/find";
                }
                keyframes.deleteRule("to");
                if (keyframes.length !== 2 || keyframes.findRule("to") !== null) return "delete";
                keyframes.name = "none";
                if (keyframes.name !== "none" || !keyframes.cssText.includes('\"none\"')) {
                    return "name";
                }
                return "ok";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn cssom_exposes_the_web_idl_interface_hierarchy_and_descriptors() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><head><style>#target { float: left; }</style></head>\
         <body><div id='target'></div></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const target = document.getElementById("target");
                const sheet = document.styleSheets[0];
                const rule = sheet.cssRules[0];
                const declarations = [target.style, getComputedStyle(target), rule.style];
                if (!(sheet instanceof StyleSheet) || !(sheet instanceof CSSStyleSheet)) {
                    return "stylesheet hierarchy";
                }
                if (!(rule instanceof CSSGroupingRule) || !(rule instanceof CSSStyleRule)) {
                    return "rule hierarchy";
                }
                if (!declarations.every(style =>
                    style instanceof CSSStyleProperties && style instanceof CSSStyleDeclaration)) {
                    return "declaration hierarchy";
                }
                if (Object.prototype.toString.call(sheet) !== "[object CSSStyleSheet]" ||
                    Object.prototype.toString.call(sheet.cssRules) !== "[object CSSRuleList]" ||
                    Object.prototype.toString.call(rule) !== "[object CSSStyleRule]" ||
                    !declarations.every(style =>
                        Object.prototype.toString.call(style) === "[object CSSStyleProperties]")) {
                    return "interface tags";
                }
                if (rule.style.parentRule !== rule) return "rule declaration parent";
                if (target.style.parentRule !== null) return "inline declaration parent";
                target.style.cssFloat = "left";
                if (target.style.cssFloat !== "left") return `cssFloat: ${target.style.cssFloat}`;
                const globalDescriptor = Object.getOwnPropertyDescriptor(globalThis, "CSSRule");
                const memberDescriptor = Object.getOwnPropertyDescriptor(
                    CSSStyleRule.prototype,
                    "selectorText",
                );
                if (globalDescriptor.enumerable || !memberDescriptor.enumerable) {
                    return "descriptors";
                }
                if (CSSRule.CHARSET_RULE !== 2 || CSSRule.MARGIN_RULE !== 9 ||
                    Object.getOwnPropertyDescriptor(CSSRule, "STYLE_RULE").writable) {
                    return "constants";
                }
                if (CSSRule.length !== 0 || CSSRuleList.length !== 0 ||
                    StyleSheetList.length !== 0 || StyleSheet.length !== 0) {
                    return "constructor lengths";
                }
                for (const operation of [
                    () => sheet.cssRules.item(),
                    () => target.style.item(),
                    () => target.style.getPropertyValue(),
                    () => target.style.getPropertyPriority(),
                    () => target.style.setProperty("color"),
                    () => target.style.removeProperty(),
                    () => sheet.replaceSync(),
                ]) {
                    try { operation(); return "missing argument check"; } catch (error) {
                        if (!(error instanceof TypeError)) return "wrong argument error";
                    }
                }
                for (const construct of [
                    () => new StyleSheet(),
                    () => new CSSRule(),
                    () => new CSSRuleList(),
                    () => new StyleSheetList(),
                    () => new MediaList(),
                    () => new CSSStyleDeclaration(),
                    () => new CSSStyleProperties(),
                ]) {
                    try { construct(); return "constructible interface"; } catch (error) {
                        if (!(error instanceof TypeError)) return "wrong constructor error";
                    }
                }
                return "ok";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn cssom_preserves_rule_identity_and_exposes_stylesheet_metadata() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><head><style id='sheet' media='screen, print' title='theme'>\
         body { width: 50%; } #target { height: 10px; }\
         </style></head><body><div id='target'></div></body></html>",
    )
    .unwrap();

    let result = page
        .eval(
            r##"(() => {
                const owner = document.getElementById("sheet");
                const sheet = owner.sheet;
                if (sheet.type !== "text/css" || sheet.ownerNode !== owner ||
                    sheet.ownerRule !== null || sheet.parentStyleSheet !== null ||
                    sheet.href !== null || sheet.title !== "theme" ||
                    sheet.media.length !== 2 || sheet.media.item(1) !== "print") return "metadata";
                owner.disabled = true;
                if (!owner.disabled || !sheet.disabled) return "disabled";
                const first = sheet.cssRules[0];
                const second = sheet.cssRules[1];
                first.marker = 1;
                second.marker = 2;
                sheet.insertRule("#inserted { color: red; }", 1);
                if (sheet.cssRules[0] !== first || sheet.cssRules[2] !== second ||
                    sheet.cssRules[0].marker !== 1 || sheet.cssRules[2].marker !== 2) return "identity";
                sheet.deleteRule(1);
                if (sheet.cssRules[0] !== first || sheet.cssRules[1] !== second) return "delete identity";
                if (sheet.addRule("#legacy", "color: blue") !== -1) return "addRule result";
                if (sheet.cssRules.length !== 3) return "addRule";
                sheet.removeRule(2);
                if (sheet.cssRules.length !== 2) return "removeRule";
                return "ok";
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn css_rule_list_reads_the_retained_rules_without_reparsing_the_stylesheet() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let rules = (0..2_000)
        .map(|index| format!(".rule-{index} {{ color: red; }}"))
        .collect::<String>();
    page.set_content(&format!(
        "<html><head><style>{rules}</style></head><body></body></html>"
    ))
    .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const rules = document.styleSheets[0].cssRules;
                if (rules.length !== 2000) return `length: ${rules.length}`;
                for (let pass = 0; pass < 3; pass++) {
                    for (let index = 0; index < rules.length; index++) {
                        if (rules[index] !== rules.item(index)) return `identity: ${index}`;
                    }
                }
                return Array.from(rules).length === 2000 ? "ok" : "iteration";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn deleting_import_rule_unlinks_its_child_stylesheet() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        r#"<html><head><style>@import "data:text/css,";</style></head><body></body></html>"#,
    )
    .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const sheet = document.styleSheets[0];
                const child = sheet.cssRules[0].styleSheet;
                if (child.parentStyleSheet !== sheet) return "missing parent";
                sheet.deleteRule(0);
                return child.parentStyleSheet === null ? "ok" : "still linked";
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn constructed_stylesheets_apply_through_document_adoption() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body><div id='target'></div></body></html>")
        .unwrap();

    let result = page
        .eval(
            r##"(() => {
                const target = document.getElementById("target");
                const sheet = new CSSStyleSheet();
                const list = document.adoptedStyleSheets;
                sheet.replaceSync("#target { color: red; }");
                document.adoptedStyleSheets = [sheet];
                if (document.adoptedStyleSheets !== list || list[0] !== sheet ||
                    getComputedStyle(target).color !== "rgb(255, 0, 0)") return "initial adoption";
                if (sheet.cssRules[0].style.color !== "red" ||
                    sheet.cssRules[0].style.getPropertyValue("color") !== "red") return "rule style getter";
                sheet.cssRules[0].style.color = "green";
                if (sheet.cssRules[0].style.color !== "green" ||
                    getComputedStyle(target).color !== "rgb(0, 128, 0)") return "rule style mutation";
                const empty = new CSSStyleSheet();
                empty.replaceSync("#target { }");
                list.push(empty);
                list[1] = sheet;
                list[0] = empty;
                if (list[0] !== empty || list[1] !== sheet ||
                    getComputedStyle(target).color !== "rgb(0, 128, 0)") return "indexed mutation";
                list.shift();
                sheet.replaceSync("#target { color: blue; }");
                if (getComputedStyle(target).color !== "rgb(0, 0, 255)") return "mutation";
                list.pop();
                if (list.length !== 0 || getComputedStyle(target).color === "rgb(0, 0, 255)") {
                    return "removal";
                }
                return "ok";
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn constructed_stylesheets_follow_adoption_order_and_disallow_imports() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body><div id='target'></div></body></html>")
        .unwrap();

    let result = page
        .eval(
            r##"(() => {
                const target = document.getElementById("target");
                const red = new CSSStyleSheet();
                const green = new CSSStyleSheet();
                red.replaceSync("#target { color: red; z-index: 1; font-style: normal; }");
                green.replaceSync("#target { color: green; z-index: 2; font-style: italic; }");
                document.adoptedStyleSheets = [red, green];
                if (getComputedStyle(target).color !== "rgb(0, 128, 0)" ||
                    getComputedStyle(target).zIndex !== "2" ||
                    getComputedStyle(target).fontStyle !== "italic") return "initial order";
                document.adoptedStyleSheets = [green, red];
                if (getComputedStyle(target).color !== "rgb(255, 0, 0)" ||
                    getComputedStyle(target).zIndex !== "1" ||
                    getComputedStyle(target).fontStyle !== "normal") return "reordered";
                document.adoptedStyleSheets = [red, green, red];
                if (getComputedStyle(target).color !== "rgb(255, 0, 0)") return "duplicate";
                const imports = new CSSStyleSheet();
                imports.replaceSync('@import "ignored.css"; #target { color: blue; }');
                if (imports.cssRules.length !== 1 || !(imports.cssRules[0] instanceof CSSStyleRule)) {
                    return "replace import";
                }
                try {
                    imports.insertRule('@import "ignored.css";');
                    return "insert import";
                } catch (error) {
                    if (!(error instanceof DOMException) || error.name !== "SyntaxError") {
                        return "wrong import error";
                    }
                }
                document.adoptedStyleSheets = [];
                return "ok";
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ok");
}
