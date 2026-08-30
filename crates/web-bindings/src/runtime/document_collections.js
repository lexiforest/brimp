function __removeNode(node) {
    if (node.parentNode) node.parentNode.removeChild(node);
}

function __replaceNode(node, values) {
    const parent = node.parentNode;
    if (!parent) return;
    for (const replacement of __nodesFromArguments(values)) {
        parent.insertBefore(replacement, node);
    }
    parent.removeChild(node);
}

class Document extends Node {
    get title() { return __callHost("title", this); }
    get URL() { return this.__URL ?? location.href; }
    get documentURI() { return this.URL; }
    get contentType() { return this.__contentType ?? "text/html"; }
    get compatMode() { return this.__compatMode ?? "CSS1Compat"; }
    get location() { return this === document ? globalThis.location : null; }
    get defaultView() { return this === document ? window : null; }
    get adoptedStyleSheets() { return __documentAdoptedStyleSheets; }
    set adoptedStyleSheets(value) { __replaceAdoptedStyleSheets(value); }
    get dir() {
        const value = __asciiLowercase(this.documentElement?.getAttribute("dir") ?? "");
        return value === "ltr" || value === "rtl" || value === "auto" ? value : "";
    }
    set dir(value) { this.documentElement?.setAttribute("dir", String(value)); }
    get hidden() { return false; }
    get visibilityState() { return "visible"; }
    get characterSet() {
        if (this.__characterSet === undefined) {
            this.__characterSet = this.querySelector("meta[charset]")?.getAttribute("charset") || "UTF-8";
        }
        return this.__characterSet;
    }
    get charset() { return this.characterSet; }
    get inputEncoding() { return this.characterSet; }
    get cookie() { return __callHost("cookie", this); }
    set cookie(value) { __callHost("setCookie", this, value); }
    get documentElement() { return __callHost("documentElement", this); }
    get head() { return __callHost("head", this); }
    get body() { return __callHost("body", this); }
    get activeElement() { return __activeElement || this.body; }
    get children() { return new HTMLCollection(() => [...this.childNodes].filter(node => node instanceof Element)); }
    get childElementCount() { return this.children.length; }
    get firstElementChild() { return this.children.item(0); }
    get lastElementChild() { return this.children.item(this.children.length - 1); }
    get implementation() { return __domImplementation; }
    get styleSheets() { return __documentStyleSheets; }
    createElement(name) { return __callHost("createElement", this, name); }
    createElementNS(namespace, qualifiedName) {
        return __callHost("createElementNS", this, namespace, qualifiedName);
    }
    createTextNode(text) { return __callHost("createTextNode", this, text); }
    createComment(data) { return __callHost("createComment", this, data); }
    createDocumentFragment() { return __callHost("createDocumentFragment", this); }
    createTreeWalker(root, whatToShow = NodeFilter.SHOW_ALL, filter = null) {
        if (!(root instanceof Node)) throw new TypeError("root must be a Node");
        return new TreeWalker(root, whatToShow, filter);
    }
    elementFromPoint(x, y) {
        if (arguments.length < 2) throw new TypeError("two coordinates are required");
        x = Number(x);
        y = Number(y);
        if (!Number.isFinite(x) || !Number.isFinite(y)) throw new TypeError("coordinates must be finite");
        return __callHost("elementFromPoint", this, x, y);
    }
    createRange() { return new Range(this); }
    getSelection() { return __selection; }
    append(...nodes) { __appendNodes(this, nodes); }
    prepend(...nodes) { __prependNodes(this, nodes); }
    replaceChildren(...nodes) { __replaceChildren(this, nodes); }
    createEvent(interfaceName) {
        switch (String(interfaceName).toLowerCase()) {
            case "event":
            case "events":
            case "htmlevents":
            case "svgevents": return new Event("");
            case "customevent": return new CustomEvent("");
            case "uievent":
            case "uievents": return new UIEvent("");
            case "mouseevent":
            case "mouseevents": return new MouseEvent("");
            case "keyboardevent": return new KeyboardEvent("");
            default: throw new DOMException("The event interface is not supported", "NotSupportedError");
        }
    }
    getElementById(id) { return __callHost("getElementById", this, id); }
    getElementsByTagName(name) {
        return new HTMLCollection(() => __callHost("getElementsByTagName", this, name));
    }
    getElementsByClassName(names) {
        return new HTMLCollection(() => __callHost("getElementsByClassName", this, names));
    }
    getElementsByName(name) {
        return new NodeList(() => __callHost("getElementsByName", this, name));
    }
    querySelector(selector) { return __querySelectorWithHas(this, selector); }
    querySelectorAll(selector) { return new NodeList(__querySelectorAllWithHas(this, selector)); }
}

class XMLDocument extends Document {}

class DOMParser {
    parseFromString(input, type) {
        if (arguments.length < 2) throw new TypeError("two arguments are required");
        input = String(input);
        type = String(type);
        if (!["text/html", "text/xml", "application/xml", "application/xhtml+xml", "image/svg+xml"].includes(type)) {
            throw new TypeError("unsupported DOMParser type");
        }
        const parsed = __callHost("domParserParse", this, input, type);
        Object.defineProperties(parsed, {
            __URL: { value: document.URL, configurable: true },
            __contentType: { value: type, configurable: true },
            __characterSet: { value: "UTF-8", configurable: true },
            __compatMode: {
                value: type === "text/html" && !/^\s*<!doctype\s+html(?:\s|>)/i.test(input)
                    ? "BackCompat"
                    : "CSS1Compat",
                configurable: true,
            },
        });
        return parsed;
    }
}

function __boundaryLength(node) {
    if (!(node instanceof Node)) throw new TypeError("boundary container must be a Node");
    return node instanceof CharacterData ? node.length : node.childNodes.length;
}

function __validateBoundary(node, offset) {
    if (!(node instanceof Node)) throw new TypeError("boundary container must be a Node");
    offset = Number(offset);
    if (!Number.isFinite(offset) || offset < 0 || Math.trunc(offset) !== offset || offset > __boundaryLength(node)) {
        throw new DOMException("The offset is outside the node", "IndexSizeError");
    }
    return offset;
}

function __nodeRoot(node) {
    while (node.parentNode) node = node.parentNode;
    return node;
}

function __boundaryOrder(aNode, aOffset, bNode, bOffset) {
    if (aNode === bNode) return Math.sign(aOffset - bOffset);
    if (__nodeRoot(aNode) !== __nodeRoot(bNode)) return null;

    let child = bNode;
    while (child.parentNode && child.parentNode !== aNode) child = child.parentNode;
    if (child.parentNode === aNode) {
        return aOffset <= aNode.childNodes.indexOf(child) ? -1 : 1;
    }

    child = aNode;
    while (child.parentNode && child.parentNode !== bNode) child = child.parentNode;
    if (child.parentNode === bNode) {
        return bOffset <= bNode.childNodes.indexOf(child) ? 1 : -1;
    }

    return aNode.compareDocumentPosition(bNode) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
}

class Range {
    constructor(document) {
        this.__document = document;
        this.__startContainer = document;
        this.__startOffset = 0;
        this.__endContainer = document;
        this.__endOffset = 0;
    }
    get startContainer() { return this.__startContainer; }
    get startOffset() { return this.__startOffset; }
    get endContainer() { return this.__endContainer; }
    get endOffset() { return this.__endOffset; }
    get collapsed() {
        return this.__startContainer === this.__endContainer && this.__startOffset === this.__endOffset;
    }
    get commonAncestorContainer() {
        const ancestors = new Set();
        for (let node = this.__startContainer; node; node = node.parentNode) ancestors.add(node);
        for (let node = this.__endContainer; node; node = node.parentNode) {
            if (ancestors.has(node)) return node;
        }
        return null;
    }
    setStart(node, offset) {
        offset = __validateBoundary(node, offset);
        const order = __boundaryOrder(node, offset, this.__endContainer, this.__endOffset);
        if (order === null || order > 0) {
            this.__endContainer = node;
            this.__endOffset = offset;
        }
        this.__startContainer = node;
        this.__startOffset = offset;
    }
    setEnd(node, offset) {
        offset = __validateBoundary(node, offset);
        const order = __boundaryOrder(node, offset, this.__startContainer, this.__startOffset);
        if (order === null || order < 0) {
            this.__startContainer = node;
            this.__startOffset = offset;
        }
        this.__endContainer = node;
        this.__endOffset = offset;
    }
    setStartBefore(node) { this.setStart(node.parentNode, node.parentNode.childNodes.indexOf(node)); }
    setStartAfter(node) { this.setStart(node.parentNode, node.parentNode.childNodes.indexOf(node) + 1); }
    setEndBefore(node) { this.setEnd(node.parentNode, node.parentNode.childNodes.indexOf(node)); }
    setEndAfter(node) { this.setEnd(node.parentNode, node.parentNode.childNodes.indexOf(node) + 1); }
    collapse(toStart = false) {
        if (toStart) {
            this.__endContainer = this.__startContainer;
            this.__endOffset = this.__startOffset;
        } else {
            this.__startContainer = this.__endContainer;
            this.__startOffset = this.__endOffset;
        }
    }
    selectNode(node) {
        const parent = node.parentNode;
        if (!parent) throw new DOMException("The node has no parent", "InvalidNodeTypeError");
        const index = parent.childNodes.indexOf(node);
        this.__startContainer = parent;
        this.__startOffset = index;
        this.__endContainer = parent;
        this.__endOffset = index + 1;
    }
    selectNodeContents(node) {
        if (!(node instanceof Node)) throw new TypeError("argument must be a Node");
        this.__startContainer = node;
        this.__startOffset = 0;
        this.__endContainer = node;
        this.__endOffset = __boundaryLength(node);
    }
    cloneRange() {
        const clone = new Range(this.__document);
        clone.__startContainer = this.__startContainer;
        clone.__startOffset = this.__startOffset;
        clone.__endContainer = this.__endContainer;
        clone.__endOffset = this.__endOffset;
        return clone;
    }
    compareBoundaryPoints(how, sourceRange) {
        if (!(sourceRange instanceof Range)) throw new TypeError("argument must be a Range");
        const pairs = {
            [Range.START_TO_START]: [this.__startContainer, this.__startOffset, sourceRange.__startContainer, sourceRange.__startOffset],
            [Range.START_TO_END]: [this.__endContainer, this.__endOffset, sourceRange.__startContainer, sourceRange.__startOffset],
            [Range.END_TO_END]: [this.__endContainer, this.__endOffset, sourceRange.__endContainer, sourceRange.__endOffset],
            [Range.END_TO_START]: [this.__startContainer, this.__startOffset, sourceRange.__endContainer, sourceRange.__endOffset],
        };
        if (!(how in pairs)) throw new DOMException("Invalid comparison mode", "NotSupportedError");
        const result = __boundaryOrder(...pairs[how]);
        if (result === null) throw new DOMException("Ranges have different roots", "WrongDocumentError");
        return result;
    }
    detach() {}
    toString() {
        if (this.__startContainer === this.__endContainer && this.__startContainer instanceof CharacterData) {
            return this.__startContainer.data.slice(this.__startOffset, this.__endOffset);
        }
        return "";
    }
}
for (const [name, value] of Object.entries({ START_TO_START: 0, START_TO_END: 1, END_TO_END: 2, END_TO_START: 3 })) {
    Object.defineProperty(Range, name, { value, enumerable: true });
    Object.defineProperty(Range.prototype, name, { value, enumerable: true });
}

class Selection {
    constructor() {
        this.__range = null;
        this.__anchorNode = null;
        this.__anchorOffset = 0;
        this.__focusNode = null;
        this.__focusOffset = 0;
    }
    get anchorNode() { return this.__anchorNode; }
    get anchorOffset() { return this.__anchorOffset; }
    get focusNode() { return this.__focusNode; }
    get focusOffset() { return this.__focusOffset; }
    get isCollapsed() {
        return this.__anchorNode === this.__focusNode && this.__anchorOffset === this.__focusOffset;
    }
    get rangeCount() { return this.__range === null ? 0 : 1; }
    get type() { return this.rangeCount === 0 ? "None" : this.isCollapsed ? "Caret" : "Range"; }
    getRangeAt(index) {
        if (index !== 0 || this.__range === null) throw new DOMException("No range exists at that index", "IndexSizeError");
        return this.__range;
    }
    addRange(range) {
        if (!(range instanceof Range)) throw new TypeError("argument must be a Range");
        if (this.__range !== null) return;
        this.__range = range;
        this.__anchorNode = range.startContainer;
        this.__anchorOffset = range.startOffset;
        this.__focusNode = range.endContainer;
        this.__focusOffset = range.endOffset;
    }
    removeAllRanges() {
        this.__range = null;
        this.__anchorNode = this.__focusNode = null;
        this.__anchorOffset = this.__focusOffset = 0;
    }
    empty() { this.removeAllRanges(); }
    collapse(node, offset = 0) {
        if (node === null) { this.removeAllRanges(); return; }
        offset = __validateBoundary(node, offset);
        const range = new Range(document);
        range.setStart(node, offset);
        range.collapse(true);
        this.removeAllRanges();
        this.addRange(range);
    }
    setPosition(node, offset = 0) { this.collapse(node, offset); }
    extend(node, offset = 0) {
        if (this.__range === null) throw new DOMException("The selection has no range", "InvalidStateError");
        offset = __validateBoundary(node, offset);
        const anchorNode = this.__anchorNode;
        const anchorOffset = this.__anchorOffset;
        const order = __boundaryOrder(anchorNode, anchorOffset, node, offset);
        if (order === null) throw new DOMException("Selection endpoints have different roots", "WrongDocumentError");
        const range = new Range(document);
        if (order <= 0) {
            range.setStart(anchorNode, anchorOffset);
            range.setEnd(node, offset);
        } else {
            range.setStart(node, offset);
            range.setEnd(anchorNode, anchorOffset);
        }
        this.__range = range;
        this.__focusNode = node;
        this.__focusOffset = offset;
    }
    collapseToStart() {
        if (!this.__range) throw new DOMException("The selection has no range", "InvalidStateError");
        this.collapse(this.__range.startContainer, this.__range.startOffset);
    }
    collapseToEnd() {
        if (!this.__range) throw new DOMException("The selection has no range", "InvalidStateError");
        this.collapse(this.__range.endContainer, this.__range.endOffset);
    }
    selectAllChildren(node) {
        const range = new Range(document);
        range.selectNodeContents(node);
        this.removeAllRanges();
        this.addRange(range);
    }
    containsNode(node, allowPartialContainment = false) {
        if (!this.__range || !(node instanceof Node) || !node.parentNode) return false;
        const parent = node.parentNode;
        const index = parent.childNodes.indexOf(node);
        const start = __boundaryOrder(this.__range.startContainer, this.__range.startOffset, parent, index);
        const end = __boundaryOrder(this.__range.endContainer, this.__range.endOffset, parent, index + 1);
        return allowPartialContainment ? start < 0 && end > 0 : start <= 0 && end >= 0;
    }
    deleteFromDocument() {}
    toString() { return this.__range ? this.__range.toString() : ""; }
}
const __selection = new Selection();

const __classLists = new WeakMap();

function __domTokenListTokens(element, attribute) {
    const value = element.getAttribute(attribute) || "";
    const tokens = value.split(/[\t\n\f\r ]+/).filter(Boolean);
    return [...new Set(tokens)];
}

function __validateDomToken(token) {
    token = String(token);
    if (token === "") throw new DOMException("The token must not be empty", "SyntaxError");
    if (/[\t\n\f\r ]/.test(token)) {
        throw new DOMException("The token must not contain ASCII whitespace", "InvalidCharacterError");
    }
    return token;
}

class DOMTokenList {
    constructor(element, attribute = "class") {
        this.__element = element;
        this.__attribute = attribute;
        return new Proxy(this, {
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
    }
    get length() { return __domTokenListTokens(this.__element, this.__attribute).length; }
    get value() { return this.__element.getAttribute(this.__attribute) || ""; }
    set value(value) { this.__element.setAttribute(this.__attribute, String(value)); }
    item(index) { return __domTokenListTokens(this.__element, this.__attribute)[Number(index)] ?? null; }
    contains(token) { return __domTokenListTokens(this.__element, this.__attribute).includes(String(token)); }
    add(...tokens) {
        tokens = tokens.map(__validateDomToken);
        const values = __domTokenListTokens(this.__element, this.__attribute);
        for (const token of tokens) if (!values.includes(token)) values.push(token);
        this.__element.setAttribute(this.__attribute, values.join(" "));
    }
    remove(...tokens) {
        tokens = tokens.map(__validateDomToken);
        const remove = new Set(tokens);
        const values = __domTokenListTokens(this.__element, this.__attribute);
        if (this.__element.getAttribute(this.__attribute) !== null) {
            this.__element.setAttribute(this.__attribute, values.filter(token => !remove.has(token)).join(" "));
        }
    }
    toggle(token, force = undefined) {
        token = __validateDomToken(token);
        const present = this.contains(token);
        if (arguments.length > 1) {
            if (Boolean(force)) { if (!present) this.add(token); return true; }
            if (present) this.remove(token);
            return false;
        }
        if (present) { this.remove(token); return false; }
        this.add(token);
        return true;
    }
    replace(token, newToken) {
        token = String(token);
        newToken = String(newToken);
        if (token === "" || newToken === "") {
            throw new DOMException("The token must not be empty", "SyntaxError");
        }
        if (/[\t\n\f\r ]/.test(token) || /[\t\n\f\r ]/.test(newToken)) {
            throw new DOMException("The token must not contain ASCII whitespace", "InvalidCharacterError");
        }
        const values = __domTokenListTokens(this.__element, this.__attribute);
        const index = values.indexOf(token);
        if (index === -1) return false;
        values[index] = newToken;
        this.__element.setAttribute(this.__attribute, [...new Set(values)].join(" "));
        return true;
    }
    supports() { throw new TypeError("classList has no supported tokens"); }
    entries() { return __domTokenListTokens(this.__element, this.__attribute).entries(); }
    keys() { return __domTokenListTokens(this.__element, this.__attribute).keys(); }
    values() { return __domTokenListTokens(this.__element, this.__attribute).values(); }
    forEach(callback, thisArg = undefined) {
        __domTokenListTokens(this.__element, this.__attribute).forEach((value, index) => callback.call(thisArg, value, index, this));
    }
    toString() { return this.value; }
    get [Symbol.toStringTag]() { return "DOMTokenList"; }
}
DOMTokenList.prototype.entries = Array.prototype.entries;
DOMTokenList.prototype.keys = Array.prototype.keys;
DOMTokenList.prototype.values = Array.prototype.values;
DOMTokenList.prototype.forEach = Array.prototype.forEach;
DOMTokenList.prototype[Symbol.iterator] = Array.prototype[Symbol.iterator];

class HTMLCollection {
    constructor(items) {
        this.__items = items;
        return new Proxy(this, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return target.item(Number(property)) ?? undefined;
                }
                if (Reflect.has(target, property)) return Reflect.get(target, property, receiver);
                if (typeof property === "string") return target.namedItem(property) ?? undefined;
                return undefined;
            },
            has(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return Number(property) < target.length;
                }
                if (Reflect.has(target, property)) return true;
                return typeof property === "string" && target.namedItem(property) !== null;
            },
            getOwnPropertyDescriptor(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    const value = target.item(Number(property));
                    return value === null ? undefined : {
                        value, configurable: true, enumerable: true, writable: false,
                    };
                }
                return Reflect.getOwnPropertyDescriptor(target, property);
            },
        });
    }
    get length() { return this.__items().length; }
    item(index) { return this.__items()[Number(index)] ?? null; }
    namedItem(name) {
        name = String(name);
        if (name === "") return null;
        return this.__items().find(element => element.id === name || element.getAttribute("name") === name) ?? null;
    }
    get [Symbol.toStringTag]() { return "HTMLCollection"; }
    [Symbol.iterator]() { return this.__items()[Symbol.iterator](); }
}

class NodeList extends Array {
    constructor(items = []) {
        if (typeof items === "number") {
            super(items);
            this.__items = null;
        } else if (typeof items === "function") {
            super();
            this.__items = items;
        } else {
            super(...items);
            this.__items = null;
        }
        if (this.__items !== null) {
            return new Proxy(this, {
                get(target, property, receiver) {
                    if (property === "length") return target.__items().length;
                    if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                        return target.item(Number(property)) ?? undefined;
                    }
                    return Reflect.get(target, property, receiver);
                },
                has(target, property) {
                    if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                        return Number(property) < target.__items().length;
                    }
                    return Reflect.has(target, property);
                },
                getOwnPropertyDescriptor(target, property) {
                    if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                        const value = target.item(Number(property));
                        return value === null ? undefined : {
                            value, configurable: true, enumerable: true, writable: false,
                        };
                    }
                    return Reflect.getOwnPropertyDescriptor(target, property);
                },
            });
        }
    }
    item(index) {
        return this.__items === null
            ? this[Number(index)] ?? null
            : this.__items()[Number(index)] ?? null;
    }
    entries() { return Array.prototype.entries.call(this); }
    keys() { return Array.prototype.keys.call(this); }
    values() { return Array.prototype.values.call(this); }
    forEach(callback, thisArg = undefined) {
        Array.prototype.forEach.call(this, callback, thisArg);
    }
    [Symbol.iterator]() { return this.values(); }
    get [Symbol.toStringTag]() { return "NodeList"; }
}

const __attrConstructionToken = {};

class Attr extends Node {
    constructor(token, element, record) {
        if (token !== __attrConstructionToken) throw new TypeError("Illegal constructor");
        super();
        this.__element = element;
        this.__record = record;
    }
    __update(record) { this.__record = record; }
    __currentRecord() {
        if (this.__element === null) return null;
        const record = __attributeRecords(this.__element).find(item =>
            item.namespaceURI === this.namespaceURI && item.localName === this.localName
        );
        if (record === undefined) this.__element = null;
        else this.__update(record);
        return record ?? null;
    }
    get namespaceURI() { return this.__record.namespaceURI; }
    get prefix() { return this.__record.prefix; }
    get localName() { return this.__record.localName; }
    get name() { return this.__record.name; }
    get value() {
        this.__currentRecord();
        return this.__record.value;
    }
    set value(value) {
        value = String(value);
        if (this.__element !== null && this.namespaceURI === null) {
            this.__element.setAttribute(this.name, value);
        }
        this.__record.value = value;
    }
    get ownerElement() {
        this.__currentRecord();
        return this.__element;
    }
    get specified() { return true; }
    get nodeType() { return Node.ATTRIBUTE_NODE; }
    get nodeName() { return this.name; }
    get nodeValue() { return this.value; }
    set nodeValue(value) { this.value = value == null ? "" : value; }
    get textContent() { return this.value; }
    set textContent(value) { this.value = value == null ? "" : value; }
    get ownerDocument() { return document; }
    get parentNode() { return null; }
    get parentElement() { return null; }
    get childNodes() { return new NodeList(); }
    get firstChild() { return null; }
    get lastChild() { return null; }
    get previousSibling() { return null; }
    get nextSibling() { return null; }
    hasChildNodes() { return false; }
}

function __attributeRecords(element) {
    return JSON.parse(__callHost("elementAttributes", element));
}

class NamedNodeMap {
    constructor(element) {
        this.__element = element;
        this.__cache = new Map();
        return new Proxy(this, {
            get(target, property, receiver) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return target.item(Number(property)) ?? undefined;
                }
                if (Reflect.has(target, property)) return Reflect.get(target, property, receiver);
                return typeof property === "string" ? target.getNamedItem(property) ?? undefined : undefined;
            },
            has(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    return Number(property) < target.length;
                }
                return Reflect.has(target, property) ||
                    (typeof property === "string" && target.getNamedItem(property) !== null);
            },
            getOwnPropertyDescriptor(target, property) {
                if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                    const value = target.item(Number(property));
                    return value === null ? undefined : {
                        value, configurable: true, enumerable: true, writable: false,
                    };
                }
                if (typeof property === "string" && !Reflect.has(target, property)) {
                    const value = target.getNamedItem(property);
                    if (value !== null) return {
                        value, configurable: true, enumerable: false, writable: false,
                    };
                }
                return Reflect.getOwnPropertyDescriptor(target, property);
            },
            ownKeys(target) {
                const records = __attributeRecords(target.__element);
                const indices = records.map((_, index) => String(index));
                const names = [];
                for (const record of records) {
                    if (record.name !== record.name.toLowerCase() ||
                        Reflect.has(target, record.name) || names.includes(record.name)) continue;
                    names.push(record.name);
                }
                return indices.concat(names);
            },
        });
    }
    __attr(record) {
        const key = `${record.namespaceURI === null ? "" : record.namespaceURI}\0${record.localName}`;
        let attr = this.__cache.get(key);
        if (!attr) {
            attr = new Attr(__attrConstructionToken, this.__element, record);
            this.__cache.set(key, attr);
        } else {
            attr.__element = this.__element;
            attr.__update(record);
        }
        return attr;
    }
    get length() { return __attributeRecords(this.__element).length; }
    item(index) {
        const record = __attributeRecords(this.__element)[Number(index)];
        return record === undefined ? null : this.__attr(record);
    }
    getNamedItem(name) {
        name = String(name).toLowerCase();
        const record = __attributeRecords(this.__element).find(item => item.name === name);
        return record === undefined ? null : this.__attr(record);
    }
    removeNamedItem(name) {
        const attr = this.getNamedItem(name);
        if (attr === null) throw new DOMException("attribute was not found", "NotFoundError");
        this.__element.removeAttribute(attr.name);
        attr.__element = null;
        return attr;
    }
    get [Symbol.toStringTag]() { return "NamedNodeMap"; }
    [Symbol.iterator]() {
        return __attributeRecords(this.__element).map(record => this.__attr(record))[Symbol.iterator]();
    }
}
