class Node extends EventTarget {
    get nodeType() { return __callHost("nodeType", this); }
    get nodeName() { return __callHost("nodeName", this); }
    get parentNode() { return __callHost("parentNode", this); }
    get ownerDocument() { return __callHost("ownerDocument", this); }
    get baseURI() { return this instanceof Document ? this.URL : this.ownerDocument?.URL ?? null; }
    get parentElement() {
        const parent = this.parentNode;
        return parent instanceof Element ? parent : null;
    }
    get firstChild() { return __callHost("firstChild", this); }
    get lastChild() { return __callHost("lastChild", this); }
    get previousSibling() { return __callHost("previousSibling", this); }
    get nextSibling() { return __callHost("nextSibling", this); }
    get childNodes() { return __callHost("childNodes", this); }
    hasChildNodes() { return this.firstChild !== null; }
    get isConnected() {
        let node = this;
        while (node.parentNode) node = node.parentNode;
        return node instanceof Document;
    }
    contains(other) {
        if (other === null) return false;
        while (other) {
            if (other === this) return true;
            other = other.parentNode;
        }
        return false;
    }
    compareDocumentPosition(other) {
        if (!(other instanceof Node)) throw new TypeError("argument must be a Node");
        if (other === this) return 0;
        const thisAncestors = [];
        const otherAncestors = [];
        for (let node = this; node; node = node.parentNode) thisAncestors.unshift(node);
        for (let node = other; node; node = node.parentNode) otherAncestors.unshift(node);
        if (thisAncestors[0] !== otherAncestors[0]) {
            if (!__disconnectedNodeOrder.has(this)) __disconnectedNodeOrder.set(this, __nextDisconnectedNodeOrder++);
            if (!__disconnectedNodeOrder.has(other)) __disconnectedNodeOrder.set(other, __nextDisconnectedNodeOrder++);
            const order = __disconnectedNodeOrder.get(this) < __disconnectedNodeOrder.get(other)
                ? Node.DOCUMENT_POSITION_FOLLOWING
                : Node.DOCUMENT_POSITION_PRECEDING;
            return Node.DOCUMENT_POSITION_DISCONNECTED | Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC | order;
        }
        if (this.contains(other)) {
            return Node.DOCUMENT_POSITION_FOLLOWING | Node.DOCUMENT_POSITION_CONTAINED_BY;
        }
        if (other.contains(this)) {
            return Node.DOCUMENT_POSITION_PRECEDING | Node.DOCUMENT_POSITION_CONTAINS;
        }
        let index = 0;
        while (thisAncestors[index] === otherAncestors[index]) index++;
        const siblings = thisAncestors[index - 1].childNodes;
        return siblings.indexOf(thisAncestors[index]) < siblings.indexOf(otherAncestors[index])
            ? Node.DOCUMENT_POSITION_FOLLOWING
            : Node.DOCUMENT_POSITION_PRECEDING;
    }
    get textContent() { return __callHost("textContent", this); }
    set textContent(value) { __callHost("setTextContent", this, value); }
    get nodeValue() {
        return this.nodeType === Node.TEXT_NODE || this.nodeType === Node.COMMENT_NODE
            ? this.textContent
            : null;
    }
    set nodeValue(value) {
        if (this.nodeType === Node.TEXT_NODE || this.nodeType === Node.COMMENT_NODE) {
            this.textContent = value == null ? "" : String(value);
        }
    }
    appendChild(child) {
        const result = __callHost("appendChild", this, child);
        if (child instanceof HTMLIFrameElement) child.__connected();
        return result;
    }
    removeChild(child) { return __callHost("removeChild", this, child); }
    insertBefore(child, reference) {
        const result = __callHost("insertBefore", this, child, reference);
        if (child instanceof HTMLIFrameElement) child.__connected();
        return result;
    }
    replaceChild(node, child) {
        if (node === child) return child;
        this.insertBefore(node, child);
        return this.removeChild(child);
    }
    cloneNode(deep = false) { return __callHost("cloneNode", this, Boolean(deep)); }
}
for (const [name, value] of Object.entries({
    ELEMENT_NODE: 1,
    ATTRIBUTE_NODE: 2,
    TEXT_NODE: 3,
    CDATA_SECTION_NODE: 4,
    ENTITY_REFERENCE_NODE: 5,
    ENTITY_NODE: 6,
    PROCESSING_INSTRUCTION_NODE: 7,
    COMMENT_NODE: 8,
    DOCUMENT_NODE: 9,
    DOCUMENT_TYPE_NODE: 10,
    DOCUMENT_FRAGMENT_NODE: 11,
    NOTATION_NODE: 12,
    DOCUMENT_POSITION_DISCONNECTED: 0x01,
    DOCUMENT_POSITION_PRECEDING: 0x02,
    DOCUMENT_POSITION_FOLLOWING: 0x04,
    DOCUMENT_POSITION_CONTAINS: 0x08,
    DOCUMENT_POSITION_CONTAINED_BY: 0x10,
    DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 0x20,
})) {
    Object.defineProperty(Node, name, { value, enumerable: true });
    Object.defineProperty(Node.prototype, name, { value, enumerable: true });
}

class DOMImplementation {
    hasFeature() { return true; }
}
const __domImplementation = new DOMImplementation();

const __validCustomElementLocalName = /^(?:[A-Za-z][^\0\t\n\f\r\u0020\/>]*|[:_\u0080-\u{10FFFF}][A-Za-z0-9-.:_\u0080-\u{10FFFF}]*)$/u;
const __reservedCustomElementNames = new Set([
    "annotation-xml", "color-profile", "font-face", "font-face-src",
    "font-face-uri", "font-face-format", "font-face-name", "missing-glyph",
]);

function __isValidCustomElementName(name) {
    if (name.length === 0 || name[0] < "a" || name[0] > "z" || !name.includes("-") ||
        __reservedCustomElementNames.has(name) || !__validCustomElementLocalName.test(name)) return false;
    for (const character of name) {
        if (character >= "A" && character <= "Z") return false;
    }
    return true;
}

class CustomElementRegistry {
    constructor() {
        this.__definitions = new Map();
        this.__constructorNames = new Map();
        this.__waiters = new Map();
    }
    define(name, constructor, options = {}) {
        name = String(name);
        if (!__isValidCustomElementName(name)) {
            throw new DOMException("Invalid custom element name", "SyntaxError");
        }
        if (typeof constructor !== "function") throw new TypeError("constructor must be callable");
        if (this.__definitions.has(name) || this.__constructorNames.has(constructor)) {
            throw new DOMException("Custom element definition already exists", "NotSupportedError");
        }
        if (options.extends !== undefined) String(options.extends);
        this.__definitions.set(name, constructor);
        this.__constructorNames.set(constructor, name);
        const waiters = this.__waiters.get(name) ?? [];
        this.__waiters.delete(name);
        for (const resolve of waiters) resolve(constructor);
    }
    get(name) { return this.__definitions.get(String(name)); }
    getName(constructor) { return this.__constructorNames.get(constructor) ?? null; }
    whenDefined(name) {
        name = String(name);
        if (!__isValidCustomElementName(name)) {
            return Promise.reject(new DOMException("Invalid custom element name", "SyntaxError"));
        }
        const constructor = this.__definitions.get(name);
        if (constructor) return Promise.resolve(constructor);
        return new Promise(resolve => {
            const waiters = this.__waiters.get(name) ?? [];
            waiters.push(resolve);
            this.__waiters.set(name, waiters);
        });
    }
    upgrade() {}
}
const customElements = new CustomElementRegistry();

function __nodesFromArguments(values) {
    return values.map(value => value instanceof Node ? value : document.createTextNode(String(value)));
}

function __appendNodes(parent, values) {
    for (const node of __nodesFromArguments(values)) parent.appendChild(node);
}

function __prependNodes(parent, values) {
    const reference = parent.firstChild;
    for (const node of __nodesFromArguments(values)) parent.insertBefore(node, reference);
}

function __replaceChildren(parent, values) {
    const nodes = __nodesFromArguments(values);
    while (parent.firstChild) parent.removeChild(parent.firstChild);
    for (const node of nodes) parent.appendChild(node);
}

