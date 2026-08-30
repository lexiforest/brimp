const __cssRuleConstructionToken = {};
const __cssRuleListConstructionToken = {};
const __styleSheetListConstructionToken = {};
const __styleSheetConstructionToken = {};
const __mediaListConstructionToken = {};
const __styleSheetsByOwner = new WeakMap();
const __cssRules = new WeakSet();
const __cssRuleLists = new WeakSet();
const __styleSheets = new WeakSet();
const __styleSheetLists = new WeakSet();
const __mediaLists = new WeakSet();
const __mediaQueryLists = new WeakSet();
const __mediaQueryListConstructionToken = {};

function __requireCssRule(value) {
    if (!__cssRules.has(value)) throw new TypeError("receiver is not a CSSRule");
}

function __requireCssRuleList(value) {
    if (!__cssRuleLists.has(value)) throw new TypeError("receiver is not a CSSRuleList");
}

function __requireStyleSheet(value) {
    if (!__styleSheets.has(value)) throw new TypeError("receiver is not a StyleSheet");
}

function __requireStyleSheetList(value) {
    if (!__styleSheetLists.has(value)) throw new TypeError("receiver is not a StyleSheetList");
}

function __requireMediaList(value) {
    if (!__mediaLists.has(value)) throw new TypeError("receiver is not a MediaList");
}

function __requireMediaQueryList(value) {
    if (!__mediaQueryLists.has(value)) throw new TypeError("receiver is not a MediaQueryList");
}

function __cssomResult(serialized) {
    const result = JSON.parse(serialized);
    if (result.error !== undefined) throw new DOMException(result.message, result.error);
    return result.value;
}

class CSSRule {
    constructor(...args) {
        const [token, cssText, parentStyleSheet] = args;
        if (token !== __cssRuleConstructionToken) throw new TypeError("Illegal constructor");
        __cssRules.add(this);
        this.__cssText = cssText;
        this.__parentStyleSheet = parentStyleSheet;
    }
    get cssText() {
        __requireCssRule(this);
        return this.__cssText;
    }
    set cssText(value) {
        __requireCssRule(this);
        this.__replaceText(String(value));
    }
    get parentRule() {
        __requireCssRule(this);
        return this.__parentRule ?? null;
    }
    get parentStyleSheet() {
        __requireCssRule(this);
        return this.__parentStyleSheet;
    }
    get type() {
        __requireCssRule(this);
        return 0;
    }
    __replaceText(text) {
        if (this.__parentRule !== null && this.__parentRule !== undefined) {
            this.__parentRule.__replaceRuleObject(this, text);
        } else {
            this.__parentStyleSheet.__replaceRuleObject(this, text);
        }
    }
}
for (const [name, value] of Object.entries({
    STYLE_RULE: 1,
    CHARSET_RULE: 2,
    IMPORT_RULE: 3,
    MEDIA_RULE: 4,
    FONT_FACE_RULE: 5,
    PAGE_RULE: 6,
    KEYFRAMES_RULE: 7,
    KEYFRAME_RULE: 8,
    MARGIN_RULE: 9,
    NAMESPACE_RULE: 10,
    SUPPORTS_RULE: 12,
})) {
    for (const target of [CSSRule, CSSRule.prototype]) {
        Object.defineProperty(target, name, {
            value,
            writable: false,
            enumerable: true,
            configurable: false,
        });
    }
}

class CSSStyleRule extends CSSRule {
    get type() { return CSSRule.STYLE_RULE; }
    get selectorText() {
        const brace = this.__cssText.indexOf("{");
        return (brace < 0 ? "" : this.__cssText.slice(0, brace)).trim();
    }
    set selectorText(value) {
        const brace = this.__cssText.indexOf("{");
        if (brace < 0) return;
        try {
            this.__replaceText(`${String(value)} ${this.__cssText.slice(brace)}`);
        } catch (error) {
            if (error instanceof DOMException && error.name === "SyntaxError") return;
            throw error;
        }
    }
    get style() {
        __requireCssRule(this);
        return this.__style ??= new CSSRuleStyleDeclaration(this);
    }
    set style(value) { this.style.cssText = String(value); }
}

function __cssRulePropertyName(name) {
    if (name === "cssFloat") return "float";
    return String(name).replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`);
}

class CSSRuleStyleDeclaration {
    constructor(rule) {
        this.__rule = rule;
        const proxy = new Proxy(this, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return target.item(Number(property)) ?? undefined;
                }
                if (typeof property === "string" && !(property in target)) {
                    return target.getPropertyValue(__cssRulePropertyName(property));
                }
                return Reflect.get(target, property, receiver);
            },
            set(target, property, value, receiver) {
                if (typeof property === "string" && !(property in target)) {
                    target.setProperty(__cssRulePropertyName(property), value);
                    return true;
                }
                return Reflect.set(target, property, value, receiver);
            },
            has(target, property) {
                if (Reflect.has(target, property)) return true;
                if (typeof property !== "string") return false;
                if (/^(0|[1-9][0-9]*)$/.test(property)) {
                    return Number(property) < target.length;
                }
                return __cssSupportsDeclaration(__cssRulePropertyName(property), "initial");
            },
        });
        __styleDeclarations.add(this);
        __styleDeclarationTargets.set(proxy, this);
        __styleDeclarationProxies.set(this, proxy);
        return proxy;
    }
    __entries() {
        return JSON.parse(__callHost("styleRuleDeclarations", window, this.__rule.cssText));
    }
    get cssText() {
        return this.__entries().map(entry =>
            `${entry[0]}: ${entry[1]}${entry[2] ? " !important" : ""};`
        ).join(" ");
    }
    set cssText(value) {
        this.__rule.__replaceText(`${this.__rule.selectorText} { ${String(value)} }`);
    }
    get length() { return this.__entries().length; }
    item(index) {
        if (arguments.length === 0) throw new TypeError("CSSStyleDeclaration.item requires an index");
        return this.__entries()[Number(index)]?.[0] ?? "";
    }
    getPropertyValue(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.getPropertyValue requires a property");
        }
        return __callHost("styleRuleGetProperty", window, this.__rule.cssText, String(name));
    }
    getPropertyPriority(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.getPropertyPriority requires a property");
        }
        const entry = this.__entries().find(entry => entry[0] === String(name));
        return entry?.[2] ? "important" : "";
    }
    setProperty(name, value, priority = "") {
        if (arguments.length < 2) {
            throw new TypeError("CSSStyleDeclaration.setProperty requires a property and value");
        }
        name = String(name);
        value = String(value);
        priority = String(priority);
        if (priority && priority.toLowerCase() !== "important") return;
        if (!value) {
            this.removeProperty(name);
            return;
        }
        const important = priority ? " !important" : "";
        this.cssText = `${this.cssText} ${name}: ${value}${important};`;
    }
    removeProperty(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.removeProperty requires a property");
        }
        name = String(name);
        const old = this.getPropertyValue(name);
        const declarations = this.__entries()
            .filter(entry => entry[0] !== name)
            .map(entry => `${entry[0]}: ${entry[1]}${entry[2] ? " !important" : ""};`)
            .join(" ");
        this.cssText = declarations;
        return old;
    }
    [Symbol.iterator]() { return this.__entries().map(entry => entry[0])[Symbol.iterator](); }
}

class CSSImportRule extends CSSRule {
    get type() { return CSSRule.IMPORT_RULE; }
    get href() {
        const match = this.cssText.match(/^@import\s+(?:url\(\s*)?(["'])(.*?)\1\s*\)?/i);
        return match ? new URL(match[2], document.baseURI).href : "";
    }
    get media() {
        if (this.__media === undefined) {
            let tail = this.cssText.replace(/^@import\s+(?:url\(\s*)?(?:["'])(.*?)(?:["'])\s*\)?/i, "");
            tail = tail.replace(/\s+supports\([\s\S]*$/i, "").replace(/;\s*$/, "").trim();
            this.__media = new MediaList(
                __mediaListConstructionToken,
                tail === "all" ? "all" : tail,
            );
        }
        return this.__media;
    }
    set media(value) { this.media.mediaText = value; }
    get supportsText() {
        const marker = this.cssText.toLowerCase().indexOf("supports(");
        if (marker < 0) return null;
        const start = marker + "supports(".length;
        let depth = 1;
        for (let index = start; index < this.cssText.length; index++) {
            if (this.cssText[index] === "(") depth++;
            else if (this.cssText[index] === ")" && --depth === 0) {
                return this.cssText.slice(start, index);
            }
        }
        return null;
    }
    get layerName() {
        const tail = this.cssText.replace(/^@import\s+(?:url\(\s*)?(?:["'])(.*?)(?:["'])\s*\)?/i, "");
        const match = tail.match(/\blayer(?:\(\s*([^)]*?)\s*\))?/i);
        return match ? (match[1] ?? "") : null;
    }
    get styleSheet() {
        __requireCssRule(this);
        if (this.__styleSheet === undefined) {
            const sheet = Object.create(CSSStyleSheet.prototype);
            __initializeStyleSheet(sheet, null, [], { importRule: this });
            sheet.__href = this.href;
            sheet.__parentStyleSheet = this.parentStyleSheet;
            this.__styleSheet = sheet;
        }
        return this.__styleSheet;
    }
    __detach() {
        if (this.__styleSheet !== undefined) this.__styleSheet.__parentStyleSheet = null;
    }
}

class CSSGroupingRule extends CSSRule {
    get cssRules() {
        __requireCssRule(this);
        if (this.__ruleList === undefined) {
            this.__ruleObjects = [];
            this.__ruleList = new CSSRuleList(__cssRuleListConstructionToken, this);
        }
        this.__setRuleTexts(__cssomResult(__callHost("nestedRuleTexts", window, this.cssText)));
        return this.__ruleList;
    }
    __setRuleTexts(texts) {
        for (let index = 0; index < texts.length; index++) {
            const text = texts[index];
            const current = this.__ruleObjects[index];
            if (current !== undefined) current.__cssText = text;
            else this.__ruleObjects[index] = __newCssRule(
                text,
                this.parentStyleSheet,
                this,
                this instanceof CSSKeyframesRule ? "keyframe" : null,
            );
        }
        this.__ruleObjects.length = texts.length;
    }
    __rules() {
        void this.cssRules;
        return this.__ruleObjects;
    }
    __replaceNestedTexts(texts) {
        const brace = this.cssText.indexOf("{");
        this.__replaceText(`${this.cssText.slice(0, brace).trim()} { ${texts.join("\n")} }`);
        this.__setRuleTexts(__cssomResult(__callHost("nestedRuleTexts", window, this.cssText)));
    }
    __replaceRuleObject(rule, text) {
        const rules = this.__rules();
        const index = rules.indexOf(rule);
        if (index < 0) throw new DOMException("The CSS rule is no longer in this group", "InvalidStateError");
        const texts = rules.map(item => item.cssText);
        texts[index] = this instanceof CSSKeyframesRule
            ? __parseKeyframeRule(text)
            : __cssomResult(__callHost("parseStyleSheetRule", window, text))[0];
        this.__replaceNestedTexts(texts);
    }
    insertRule(rule, index = 0) {
        if (arguments.length === 0) throw new TypeError("CSSGroupingRule.insertRule requires a rule");
        const rules = this.__rules();
        index = __toUnsignedLong(index);
        if (index > rules.length) throw new DOMException("Rule index is out of range", "IndexSizeError");
        rule = String(rule);
        if (/^\s*@(import|namespace)\b/i.test(rule)) {
            throw new DOMException("This rule is not allowed in a grouping rule", "HierarchyRequestError");
        }
        const parsed = __cssomResult(__callHost("parseStyleSheetRule", window, rule))[0];
        const texts = rules.map(item => item.cssText);
        texts.splice(index, 0, parsed);
        this.__replaceNestedTexts(texts);
        return index;
    }
    deleteRule(index) {
        if (arguments.length === 0) throw new TypeError("CSSGroupingRule.deleteRule requires an index");
        const rules = this.__rules();
        index = __toUnsignedLong(index);
        if (index >= rules.length) throw new DOMException("Rule index is out of range", "IndexSizeError");
        const removed = rules[index];
        const texts = rules.map(item => item.cssText);
        texts.splice(index, 1);
        this.__replaceNestedTexts(texts);
        removed.__detach?.();
    }
}

Object.setPrototypeOf(CSSStyleRule.prototype, CSSGroupingRule.prototype);
Object.setPrototypeOf(CSSStyleRule, CSSGroupingRule);

class CSSConditionRule extends CSSGroupingRule {
    get conditionText() {
        __requireCssRule(this);
        const brace = this.cssText.indexOf("{");
        const prelude = (brace < 0 ? this.cssText : this.cssText.slice(0, brace)).trim();
        return prelude.replace(/^@(media|supports)\s+/i, "");
    }
    set conditionText(_value) { __requireCssRule(this); }
}
class CSSMediaRule extends CSSConditionRule {
    get type() { return CSSRule.MEDIA_RULE; }
    get media() {
        __requireCssRule(this);
        if (this.__media === undefined) {
            this.__media = new MediaList(
                __mediaListConstructionToken,
                this.conditionText,
                mediaText => {
                    const brace = this.cssText.indexOf("{");
                    if (brace >= 0) this.__replaceText(`@media ${mediaText} ${this.cssText.slice(brace)}`);
                },
            );
        }
        return this.__media;
    }
    set media(value) { this.media.mediaText = value; }
}
class CSSFontFaceRule extends CSSRule { get type() { return CSSRule.FONT_FACE_RULE; } }
class CSSPageRule extends CSSRule { get type() { return CSSRule.PAGE_RULE; } }
function __parseKeyframeRule(rule) {
    const rules = __cssomResult(__callHost(
        "nestedRuleTexts",
        window,
        `@keyframes __brimp { ${String(rule)} }`,
    ));
    if (rules.length !== 1) throw new DOMException("The keyframe rule is invalid", "SyntaxError");
    return rules[0];
}
function __keyframeSelector(rule) {
    const brace = rule.cssText.indexOf("{");
    return (brace < 0 ? rule.cssText : rule.cssText.slice(0, brace)).trim();
}
function __serializeKeyframesName(value) {
    value = String(value);
    if (/^(?:initial|inherit|unset|revert|revert-layer|none)$/i.test(value)) {
        return `"${value.replace(/["\\]/g, "\\$&")}"`;
    }
    return value;
}
class CSSKeyframesRule extends CSSGroupingRule {
    get type() { return CSSRule.KEYFRAMES_RULE; }
    get name() {
        __requireCssRule(this);
        const brace = this.cssText.indexOf("{");
        const prelude = (brace < 0 ? this.cssText : this.cssText.slice(0, brace)).trim();
        const serialized = prelude.replace(/^@(?:-webkit-)?keyframes\s+/i, "");
        if (serialized.startsWith('"') && serialized.endsWith('"')) {
            return serialized.slice(1, -1).replace(/\\(["\\])/g, "$1");
        }
        return serialized;
    }
    set name(value) {
        __requireCssRule(this);
        const brace = this.cssText.indexOf("{");
        if (brace < 0) return;
        this.__replaceText(`@keyframes ${__serializeKeyframesName(value)} ${this.cssText.slice(brace)}`);
    }
    get length() { return this.__rules().length; }
    appendRule(rule) {
        if (arguments.length === 0) throw new TypeError("CSSKeyframesRule.appendRule requires a rule");
        const texts = this.__rules().map(item => item.cssText);
        texts.push(__parseKeyframeRule(rule));
        this.__replaceNestedTexts(texts);
    }
    findRule(select) {
        if (arguments.length === 0) throw new TypeError("CSSKeyframesRule.findRule requires a selector");
        let selector;
        try { selector = __keyframeSelector({ cssText: __parseKeyframeRule(`${String(select)} {}`) }); }
        catch (_) { return null; }
        const rules = this.__rules();
        for (let index = rules.length - 1; index >= 0; index--) {
            if (__keyframeSelector(rules[index]) === selector) return rules[index];
        }
        return null;
    }
    deleteRule(select) {
        if (arguments.length === 0) throw new TypeError("CSSKeyframesRule.deleteRule requires a selector");
        const rule = this.findRule(select);
        if (rule === null) return;
        const texts = this.__rules().filter(item => item !== rule).map(item => item.cssText);
        this.__replaceNestedTexts(texts);
    }
}
class CSSKeyframeRule extends CSSStyleRule {
    get type() { return CSSRule.KEYFRAME_RULE; }
    get keyText() { return this.selectorText; }
    set keyText(value) {
        try {
            this.__replaceText(__parseKeyframeRule(`${String(value)} { ${this.style.cssText} }`));
        } catch (_) {}
    }
    get style() { return super.style; }
    set style(value) { super.style = value; }
}
class CSSNamespaceRule extends CSSRule {
    get type() { return CSSRule.NAMESPACE_RULE; }
    get namespaceURI() {
        return this.cssText.match(/^(?:\s*)@namespace(?:\s+[^\s"']+)?\s+(?:url\(\s*)?["']?([^"')\s;]+)["']?\s*\)?/i)?.[1] ?? "";
    }
    get prefix() {
        return this.cssText.match(/^(?:\s*)@namespace\s+([^\s"']+)\s+/i)?.[1] ?? "";
    }
}
class CSSSupportsRule extends CSSConditionRule { get type() { return CSSRule.SUPPORTS_RULE; } }

class CSSRuleList {
    constructor(...args) {
        const [token, sheet] = args;
        if (token !== __cssRuleListConstructionToken) throw new TypeError("Illegal constructor");
        __cssRuleLists.add(this);
        this.__sheet = sheet;
        const proxy = new Proxy(this, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return target.item(Number(property)) ?? undefined;
                }
                return Reflect.get(target, property, receiver);
            },
            has(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return Number(property) < target.length;
                }
                return Reflect.has(target, property);
            },
        });
        __cssRuleLists.add(proxy);
        return proxy;
    }
    get length() {
        __requireCssRuleList(this);
        return this.__sheet.__ruleObjects.length;
    }
    item(index) {
        __requireCssRuleList(this);
        if (arguments.length === 0) throw new TypeError("CSSRuleList.item requires an index");
        return this.__sheet.__ruleObjects[Number(index)] ?? null;
    }
    [Symbol.iterator]() { return this.__sheet.__ruleObjects[Symbol.iterator](); }
}

function __newCssRule(cssText, sheet, parentRule = null, forcedKind = null) {
    const trimmed = cssText.trimStart();
    const lower = trimmed.toLowerCase();
    let prototype = forcedKind === "keyframe" ? CSSKeyframeRule.prototype : CSSStyleRule.prototype;
    if (lower.startsWith("@import")) prototype = CSSImportRule.prototype;
    else if (lower.startsWith("@media")) prototype = CSSMediaRule.prototype;
    else if (lower.startsWith("@font-face")) prototype = CSSFontFaceRule.prototype;
    else if (lower.startsWith("@page")) prototype = CSSPageRule.prototype;
    else if (lower.startsWith("@keyframes") || lower.startsWith("@-webkit-keyframes")) prototype = CSSKeyframesRule.prototype;
    else if (lower.startsWith("@namespace")) prototype = CSSNamespaceRule.prototype;
    else if (lower.startsWith("@supports")) prototype = CSSSupportsRule.prototype;
    else if (lower.startsWith("@")) prototype = CSSRule.prototype;
    const rule = Object.create(prototype);
    __cssRules.add(rule);
    rule.__cssText = cssText;
    rule.__parentStyleSheet = sheet;
    rule.__parentRule = parentRule;
    if (prototype === CSSKeyframesRule.prototype) {
        const proxy = new Proxy(rule, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return receiver.cssRules.item(Number(property)) ?? undefined;
                }
                return Reflect.get(target, property, receiver);
            },
            has(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return Number(property) < target.length;
                }
                return Reflect.has(target, property);
            },
        });
        __cssRules.add(proxy);
        return proxy;
    }
    return rule;
}

class MediaList {
    constructor(...args) {
        const [token, text = "", onChange = null] = args;
        if (token !== __mediaListConstructionToken) throw new TypeError("Illegal constructor");
        __mediaLists.add(this);
        this.__items = __parseMediaQueryList(text);
        this.__onChange = onChange;
        const proxy = new Proxy(this, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return target.item(Number(property)) ?? undefined;
                }
                return Reflect.get(target, property, receiver);
            },
            has(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return Number(property) < target.length;
                }
                return Reflect.has(target, property);
            },
        });
        __mediaLists.add(proxy);
        return proxy;
    }
    get mediaText() {
        __requireMediaList(this);
        return this.__items.join(", ");
    }
    set mediaText(value) {
        __requireMediaList(this);
        this.__items = __parseMediaQueryList(value);
        this.__onChange?.(this.mediaText);
    }
    get length() {
        __requireMediaList(this);
        return this.__items.length;
    }
    item(index) {
        __requireMediaList(this);
        if (arguments.length === 0) throw new TypeError("MediaList.item requires an index");
        return this.__items[Number(index)] ?? null;
    }
    appendMedium(value) {
        __requireMediaList(this);
        if (arguments.length === 0) throw new TypeError("MediaList.appendMedium requires a medium");
        value = __parseSingleMediaQuery(value);
        if (value === null || this.__items.includes(value)) return;
        this.__items.push(value);
        this.__onChange?.(this.mediaText);
    }
    deleteMedium(value) {
        __requireMediaList(this);
        if (arguments.length === 0) throw new TypeError("MediaList.deleteMedium requires a medium");
        value = __parseSingleMediaQuery(value);
        const originalLength = this.__items.length;
        if (value !== null) this.__items = this.__items.filter(item => item !== value);
        if (this.__items.length === originalLength) {
            throw new DOMException("The medium was not found", "NotFoundError");
        }
        this.__onChange?.(this.mediaText);
    }
    toString() {
        __requireMediaList(this);
        return this.mediaText;
    }
    [Symbol.iterator]() { return this.__items[Symbol.iterator](); }
}

function __splitMediaQueries(value, separator) {
    const parts = [];
    let start = 0;
    let depth = 0;
    for (let index = 0; index < value.length; index++) {
        const character = value[index];
        if (character === "(") depth++;
        else if (character === ")" && depth > 0) depth--;
        else if (depth === 0 && separator(value, index)) {
            parts.push(value.slice(start, index));
            index += separator.width - 1;
            start = index + 1;
        }
    }
    parts.push(value.slice(start));
    return parts;
}

function __mediaLength(value) {
    const match = /^(-?(?:\d+\.?\d*|\.\d+))(px|em|rem)?$/i.exec(value.trim());
    if (match === null) return null;
    const number = Number(match[1]);
    return /^(em|rem)$/i.test(match[2] ?? "") ? number * 16 : number;
}

function __mediaResolution(value) {
    const match = /^(\d+\.?\d*|\.\d+)(dppx|dpi|dpcm)$/i.exec(value.trim());
    if (match === null) return null;
    const number = Number(match[1]);
    if (match[2].toLowerCase() === "dpi") return number / 96;
    if (match[2].toLowerCase() === "dpcm") return number * 2.54 / 96;
    return number;
}

function __matchesMediaFeature(expression) {
    const colon = expression.indexOf(":");
    const name = (colon < 0 ? expression : expression.slice(0, colon)).trim().toLowerCase();
    const value = colon < 0 ? "" : expression.slice(colon + 1).trim().toLowerCase();
    const width = Number(globalThis.innerWidth);
    const height = Number(globalThis.innerHeight);
    const numericFeatures = {
        width,
        height,
        "device-width": width,
        "device-height": height,
    };
    const numericName = name.replace(/^(min|max)-/, "");
    if (Object.hasOwn(numericFeatures, numericName)) {
        if (value === "") return numericFeatures[numericName] > 0;
        const expected = __mediaLength(value);
        if (expected === null) return false;
        if (name.startsWith("min-")) return numericFeatures[numericName] >= expected;
        if (name.startsWith("max-")) return numericFeatures[numericName] <= expected;
        return numericFeatures[numericName] === expected;
    }
    if (["resolution", "min-resolution", "max-resolution"].includes(name)) {
        const expected = __mediaResolution(value);
        if (expected === null) return false;
        const actual = Number(globalThis.devicePixelRatio);
        if (name === "min-resolution") return actual >= expected;
        if (name === "max-resolution") return actual <= expected;
        return actual === expected;
    }
    const defaults = {
        "prefers-color-scheme": "light",
        "prefers-reduced-motion": "no-preference",
        "prefers-contrast": "no-preference",
        "forced-colors": "none",
        "inverted-colors": "none",
        "color-gamut": "srgb",
        "display-mode": "browser",
        hover: "hover",
        "any-hover": "hover",
        pointer: "fine",
        "any-pointer": "fine",
        orientation: width >= height ? "landscape" : "portrait",
        update: "fast",
        scripting: "enabled",
    };
    if (Object.hasOwn(defaults, name)) return value === "" || value === defaults[name];
    if (name === "color") return value === "" || Number(value) === 8;
    if (name === "monochrome") return Number(value || 0) === 0;
    return false;
}

function __matchesSingleMediaQuery(query) {
    query = query.trim().toLowerCase().replace(/\s+/g, " ");
    let negate = false;
    if (query.startsWith("not ")) {
        negate = true;
        query = query.slice(4).trim();
    }
    if (query.startsWith("only ")) query = query.slice(5).trim();
    const andSeparator = (value, index) => /^\s+and\s+/i.test(value.slice(index));
    andSeparator.width = 5;
    const parts = __splitMediaQueries(query, andSeparator).map(part => part.trim()).filter(Boolean);
    let matches = true;
    if (parts.length !== 0 && !parts[0].startsWith("(")) {
        const mediaType = parts.shift();
        matches = mediaType === "all" || mediaType === "screen";
    }
    matches &&= parts.every(part => part.startsWith("(") && part.endsWith(")") &&
        __matchesMediaFeature(part.slice(1, -1)));
    return negate ? !matches : matches;
}

function __matchesMediaQueryList(media) {
    const commaSeparator = (value, index) => value[index] === ",";
    commaSeparator.width = 1;
    const queries = __splitMediaQueries(media, commaSeparator);
    return queries.some(query => __matchesSingleMediaQuery(query));
}

class MediaQueryListEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        this.media = String(options.media ?? "");
        this.matches = Boolean(options.matches);
    }
}

class MediaQueryList extends EventTarget {
    constructor(token, media) {
        if (token !== __mediaQueryListConstructionToken) throw new TypeError("Illegal constructor");
        super();
        __mediaQueryLists.add(this);
        this.__media = String(media);
        this.onchange = null;
    }
    get media() {
        __requireMediaQueryList(this);
        return this.__media;
    }
    get matches() {
        __requireMediaQueryList(this);
        return __matchesMediaQueryList(this.__media);
    }
    addListener(callback) {
        __requireMediaQueryList(this);
        this.addEventListener("change", callback);
    }
    removeListener(callback) {
        __requireMediaQueryList(this);
        this.removeEventListener("change", callback);
    }
}

function matchMedia(query) {
    if (arguments.length === 0) throw new TypeError("matchMedia requires a query");
    return new MediaQueryList(__mediaQueryListConstructionToken, String(query));
}

function __parseSingleMediaQuery(value) {
    value = String(value).trim();
    if (value === "" || value.includes(",")) return null;
    return value.replace(/\s+/g, " ").replace(/\s*:\s*/g, ": ");
}

function __parseMediaQueryList(value) {
    if (value === null) return [];
    return String(value)
        .split(",")
        .map(__parseSingleMediaQuery)
        .filter(value => value !== null);
}

function __initializeStyleSheet(sheet, ownerNode, ruleTexts, options = {}) {
    __styleSheets.add(sheet);
    sheet.__ownerNode = ownerNode;
    sheet.__importRule = options.importRule ?? null;
    sheet.__constructed = Boolean(options.constructed);
    sheet.__disabled = Boolean(options.disabled);
    sheet.__media = new MediaList(
        __mediaListConstructionToken,
        options.media ?? ownerNode?.getAttribute("media") ?? "",
    );
    sheet.__ruleObjects = [];
    sheet.__ruleList = new CSSRuleList(__cssRuleListConstructionToken, sheet);
    sheet.__setRuleTexts(ruleTexts);
    return sheet;
}

function __withoutConstructedImports(css) {
    let output = "";
    let position = 0;
    let depth = 0;
    let quote = null;
    while (position < css.length) {
        const character = css[position];
        if (quote !== null) {
            output += character;
            position++;
            if (character === "\\" && position < css.length) output += css[position++];
            else if (character === quote) quote = null;
            continue;
        }
        if (css.startsWith("/*", position)) {
            const end = css.indexOf("*/", position + 2);
            const next = end < 0 ? css.length : end + 2;
            output += css.slice(position, next);
            position = next;
            continue;
        }
        if (character === '"' || character === "'") {
            quote = character;
            output += character;
            position++;
            continue;
        }
        if (character === "{") depth++;
        else if (character === "}" && depth > 0) depth--;
        if (depth !== 0 || !/^@import\b/i.test(css.slice(position))) {
            output += character;
            position++;
            continue;
        }
        let cursor = position + "@import".length;
        let importQuote = null;
        let parentheses = 0;
        while (cursor < css.length) {
            const importCharacter = css[cursor++];
            if (importQuote !== null) {
                if (importCharacter === "\\") cursor++;
                else if (importCharacter === importQuote) importQuote = null;
                continue;
            }
            if (importCharacter === '"' || importCharacter === "'") importQuote = importCharacter;
            else if (importCharacter === "(") parentheses++;
            else if (importCharacter === ")" && parentheses > 0) parentheses--;
            else if (importCharacter === ";" && parentheses === 0) break;
        }
        position = cursor;
    }
    return output;
}

class StyleSheet {
    constructor(...args) {
        if (args[0] !== __styleSheetConstructionToken) throw new TypeError("Illegal constructor");
    }
    get type() {
        __requireStyleSheet(this);
        return "text/css";
    }
    get disabled() {
        __requireStyleSheet(this);
        return this.__disabled;
    }
    set disabled(value) {
        __requireStyleSheet(this);
        this.__disabled = Boolean(value);
        this.__syncAdoptedRules?.();
    }
    get ownerNode() {
        __requireStyleSheet(this);
        return this.__ownerNode;
    }
    get parentStyleSheet() {
        __requireStyleSheet(this);
        return this.__parentStyleSheet ?? null;
    }
    get href() {
        __requireStyleSheet(this);
        if (this.__href !== undefined) return this.__href;
        return this.__ownerNode instanceof HTMLLinkElement ? this.__ownerNode.href : null;
    }
    get title() {
        __requireStyleSheet(this);
        if (this.__ownerNode === null) return null;
        return this.__ownerNode.getAttribute("title") || null;
    }
    get media() {
        __requireStyleSheet(this);
        return this.__media;
    }
    set media(value) {
        __requireStyleSheet(this);
        this.__media.mediaText = value;
        this.__syncAdoptedRules?.();
    }
}

class CSSStyleSheet extends StyleSheet {
    constructor(options = {}) {
        super(__styleSheetConstructionToken);
        __initializeStyleSheet(this, null, [], { ...(options ?? {}), constructed: true });
    }
    get ownerRule() {
        __requireStyleSheet(this);
        return this.__importRule;
    }
    get cssRules() {
        __requireStyleSheet(this);
        this.__refresh();
        return this.__ruleList;
    }
    get rules() { return this.cssRules; }
    __refresh() {
        if (this.__ownerNode !== null) {
            this.__setRuleTexts(__cssomResult(__callHost("styleSheetRules", this.__ownerNode)));
        }
    }
    __setRuleTexts(ruleTexts) {
        for (let index = 0; index < ruleTexts.length; index++) {
            const text = ruleTexts[index];
            const current = this.__ruleObjects[index];
            const styleRule = !text.trimStart().startsWith("@");
            if (current && (current instanceof CSSStyleRule) === styleRule) {
                current.__cssText = text;
            } else {
                this.__ruleObjects[index] = __newCssRule(text, this);
            }
        }
        this.__ruleObjects.length = ruleTexts.length;
    }
    __rules() {
        this.__refresh();
        return this.__ruleObjects;
    }
    __syncAdoptedRules() {
        if (this.__constructed && __documentAdoptedStyleSheets.includes(this)) {
            __syncDocumentAdoptedStyleSheets();
        }
    }
    __replaceRuleObject(rule, text) {
        this.__refresh();
        const index = this.__ruleObjects.indexOf(rule);
        if (index < 0) throw new DOMException("The CSS rule is no longer in this sheet", "InvalidStateError");
        const texts = this.__ownerNode === null
            ? (() => {
                const next = this.__ruleObjects.map(item => item.cssText);
                next[index] = __cssomResult(__callHost("parseStyleSheetRule", window, text))[0];
                return next;
            })()
            : __cssomResult(__callHost("styleSheetReplaceRule", this.__ownerNode, text, index));
        for (let position = 0; position < texts.length; position++) {
            this.__ruleObjects[position].__cssText = texts[position];
        }
        this.__syncAdoptedRules();
    }
    insertRule(rule, index = 0) {
        if (arguments.length === 0) throw new TypeError("CSSStyleSheet.insertRule requires a rule");
        rule = String(rule);
        index = __toUnsignedLong(index);
        this.__refresh();
        let texts;
        if (this.__ownerNode !== null) {
            texts = __cssomResult(__callHost("styleSheetInsertRule", this.__ownerNode, rule, index));
        } else {
            if (index > this.__ruleObjects.length) {
                throw new DOMException("Rule index is out of range", "IndexSizeError");
            }
            if (/^\s*@import\b/i.test(rule)) {
                throw new DOMException("Constructed stylesheets cannot contain @import", "SyntaxError");
            }
            texts = this.__ruleObjects.map(rule => rule.cssText);
            texts.splice(index, 0, __cssomResult(__callHost("parseStyleSheetRule", window, rule))[0]);
        }
        this.__ruleObjects.splice(index, 0, __newCssRule(texts[index], this));
        for (let position = 0; position < texts.length; position++) {
            this.__ruleObjects[position].__cssText = texts[position];
        }
        this.__syncAdoptedRules();
        return index;
    }
    deleteRule(index) {
        if (arguments.length === 0) throw new TypeError("CSSStyleSheet.deleteRule requires an index");
        index = __toUnsignedLong(index);
        this.__refresh();
        let texts;
        if (this.__ownerNode !== null) {
            texts = __cssomResult(__callHost("styleSheetDeleteRule", this.__ownerNode, index));
        } else {
            if (index >= this.__ruleObjects.length) {
                throw new DOMException("Rule index is out of range", "IndexSizeError");
            }
            texts = this.__ruleObjects.map(rule => rule.cssText);
            texts.splice(index, 1);
        }
        const [removed] = this.__ruleObjects.splice(index, 1);
        removed.__detach?.();
        for (let position = 0; position < texts.length; position++) {
            this.__ruleObjects[position].__cssText = texts[position];
        }
        this.__syncAdoptedRules();
    }
    addRule(selector = "undefined", style = "undefined", index = this.cssRules.length) {
        this.insertRule(`${String(selector)} { ${String(style)} }`, index);
        return -1;
    }
    removeRule(index = 0) { this.deleteRule(index); }
    replaceSync(text) {
        __requireStyleSheet(this);
        if (arguments.length === 0) throw new TypeError("CSSStyleSheet.replaceSync requires text");
        text = String(text);
        if (this.__constructed) text = __withoutConstructedImports(text);
        const texts = this.__ownerNode === null
            ? __cssomResult(__callHost("parseStyleSheetText", window, text))
            : __cssomResult(__callHost("styleSheetReplace", this.__ownerNode, text));
        this.__setRuleTexts(texts);
        this.__syncAdoptedRules();
    }
    replace(text) {
        if (arguments.length === 0) {
            return Promise.reject(new TypeError("CSSStyleSheet.replace requires text"));
        }
        try {
            this.replaceSync(text);
            return Promise.resolve(this);
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

function __styleSheetForOwner(ownerNode) {
    let sheet = __styleSheetsByOwner.get(ownerNode);
    if (sheet !== undefined) return sheet;
    const result = JSON.parse(__callHost("styleSheetRules", ownerNode));
    if (result.error !== undefined) return null;
    sheet = __initializeStyleSheet(Object.create(CSSStyleSheet.prototype), ownerNode, result.value);
    __styleSheetsByOwner.set(ownerNode, sheet);
    return sheet;
}

class StyleSheetList {
    constructor(...args) {
        const [token] = args;
        if (token !== __styleSheetListConstructionToken) throw new TypeError("Illegal constructor");
        __styleSheetLists.add(this);
        const proxy = new Proxy(this, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return target.item(Number(property)) ?? undefined;
                }
                return Reflect.get(target, property, receiver);
            },
        });
        __styleSheetLists.add(proxy);
        return proxy;
    }
    __items() { return __callHost("styleSheetElements", document).map(__styleSheetForOwner); }
    get length() {
        __requireStyleSheetList(this);
        return this.__items().length;
    }
    item(index) {
        __requireStyleSheetList(this);
        if (arguments.length === 0) throw new TypeError("StyleSheetList.item requires an index");
        return this.__items()[Number(index)] ?? null;
    }
    [Symbol.iterator]() { return this.__items()[Symbol.iterator](); }
}
const __documentStyleSheets = new StyleSheetList(__styleSheetListConstructionToken);
let __documentAdoptedStyleSheetOwner = null;
const __documentAdoptedStyleSheetBacking = [];
let __replacingDocumentAdoptedStyleSheets = false;
const __documentAdoptedStyleSheets = new Proxy(__documentAdoptedStyleSheetBacking, {
    set(target, property, value) {
        if (__replacingDocumentAdoptedStyleSheets) return Reflect.set(target, property, value);
        const next = target.slice();
        Reflect.set(next, property, value);
        __replaceAdoptedStyleSheets(next);
        return true;
    },
    deleteProperty(target, property) {
        if (__replacingDocumentAdoptedStyleSheets) return Reflect.deleteProperty(target, property);
        const next = target.slice();
        Reflect.deleteProperty(next, property);
        __replaceAdoptedStyleSheets(next);
        return true;
    },
});

function __validateAdoptedStyleSheets(sheets) {
    const values = Array.from(sheets);
    for (const sheet of values) {
        if (!(sheet instanceof CSSStyleSheet)) {
            throw new TypeError("adoptedStyleSheets entries must be CSSStyleSheet objects");
        }
        if (!sheet.__constructed) {
            throw new DOMException("Only constructed stylesheets can be adopted", "NotAllowedError");
        }
    }
    return values;
}

function __adoptedStyleSheetCss(sheet) {
    if (sheet.__disabled) return "";
    let css = sheet.__ruleObjects.map(rule => rule.cssText).join("\n");
    if (css && sheet.__media.mediaText) css = `@media ${sheet.__media.mediaText} { ${css} }`;
    return css;
}

function __syncDocumentAdoptedStyleSheets() {
    __documentAdoptedStyleSheetOwner ??= document.createElement("style");
    const css = __documentAdoptedStyleSheetBacking.map(__adoptedStyleSheetCss).join("\n");
    __cssomResult(__callHost("styleSheetReplace", __documentAdoptedStyleSheetOwner, css));
}

function __replaceAdoptedStyleSheets(values) {
    values = __validateAdoptedStyleSheets(values);
    __replacingDocumentAdoptedStyleSheets = true;
    Array.prototype.splice.call(
        __documentAdoptedStyleSheetBacking,
        0,
        __documentAdoptedStyleSheetBacking.length,
        ...values,
    );
    __replacingDocumentAdoptedStyleSheets = false;
    __syncDocumentAdoptedStyleSheets();
}

for (const method of ["copyWithin", "fill", "pop", "push", "reverse", "shift", "sort", "splice", "unshift"]) {
    Object.defineProperty(__documentAdoptedStyleSheets, method, {
        configurable: true,
        writable: true,
        value(...args) {
            const next = this.slice();
            const result = Array.prototype[method].apply(next, args);
            __replaceAdoptedStyleSheets(next);
            return result;
        },
    });
}
