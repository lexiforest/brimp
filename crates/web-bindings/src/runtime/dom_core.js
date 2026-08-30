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
    normalize() {
        let previousText = null;
        for (const child of Array.from(this.childNodes)) {
            if (child.nodeType === Node.TEXT_NODE) {
                if (child.textContent === "") {
                    this.removeChild(child);
                } else if (previousText) {
                    previousText.textContent += child.textContent;
                    this.removeChild(child);
                } else {
                    previousText = child;
                }
            } else {
                previousText = null;
                child.normalize();
            }
        }
    }
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

const NodeFilter = Object.freeze({
    FILTER_ACCEPT: 1,
    FILTER_REJECT: 2,
    FILTER_SKIP: 3,
    SHOW_ALL: 0xFFFFFFFF,
    SHOW_ELEMENT: 0x1,
    SHOW_ATTRIBUTE: 0x2,
    SHOW_TEXT: 0x4,
    SHOW_CDATA_SECTION: 0x8,
    SHOW_ENTITY_REFERENCE: 0x10,
    SHOW_ENTITY: 0x20,
    SHOW_PROCESSING_INSTRUCTION: 0x40,
    SHOW_COMMENT: 0x80,
    SHOW_DOCUMENT: 0x100,
    SHOW_DOCUMENT_TYPE: 0x200,
    SHOW_DOCUMENT_FRAGMENT: 0x400,
    SHOW_NOTATION: 0x800,
});

class TreeWalker {
    constructor(root, whatToShow = NodeFilter.SHOW_ALL, filter = null) {
        this.root = root;
        this.whatToShow = Number(whatToShow) >>> 0;
        this.filter = filter;
        this.currentNode = root;
    }
    __decision(node) {
        const shown = (this.whatToShow & (1 << (node.nodeType - 1))) !== 0;
        if (!shown) return NodeFilter.FILTER_SKIP;
        if (this.filter === null) return NodeFilter.FILTER_ACCEPT;
        const decision = typeof this.filter === "function"
            ? this.filter(node)
            : this.filter.acceptNode(node);
        return Number(decision);
    }
    nextNode() {
        let node = this.currentNode;
        let prune = false;
        while (true) {
            if (!prune && node.firstChild) {
                node = node.firstChild;
            } else {
                while (node && node !== this.root && !node.nextSibling) node = node.parentNode;
                if (!node || node === this.root) return null;
                node = node.nextSibling;
            }
            prune = false;
            const decision = this.__decision(node);
            if (decision === NodeFilter.FILTER_ACCEPT) {
                this.currentNode = node;
                return node;
            }
            prune = decision === NodeFilter.FILTER_REJECT;
        }
    }
}

class DOMImplementation {
    hasFeature() { return true; }
}
const __domImplementation = new DOMImplementation();

function __splitSelectorGroups(selector) {
    const groups = [];
    let start = 0;
    let depth = 0;
    let quote = "";
    for (let index = 0; index < selector.length; index++) {
        const character = selector[index];
        if (quote) {
            if (character === quote && selector[index - 1] !== "\\") quote = "";
        } else if (character === '"' || character === "'") {
            quote = character;
        } else if (character === "(" || character === "[") {
            depth++;
        } else if (character === ")" || character === "]") {
            depth--;
        } else if (character === "," && depth === 0) {
            groups.push(selector.slice(start, index).trim());
            start = index + 1;
        }
    }
    groups.push(selector.slice(start).trim());
    return groups;
}

function __selectorGroupWithHas(group) {
    const predicates = [];
    let base = "";
    let offset = 0;
    while (offset < group.length) {
        const direct = group.indexOf(":has(", offset);
        const negated = group.indexOf(":not(:has(", offset);
        let start = direct;
        let isNegated = false;
        if (negated !== -1 && (direct === -1 || negated <= direct)) {
            start = negated;
            isNegated = true;
        }
        if (start === -1) {
            base += group.slice(offset);
            break;
        }
        base += group.slice(offset, start);
        const contentStart = start + (isNegated ? 10 : 5);
        let depth = 1;
        let end = contentStart;
        for (; end < group.length && depth > 0; end++) {
            if (group[end] === "(") depth++;
            else if (group[end] === ")") depth--;
        }
        if (depth !== 0 || (isNegated && group[end] !== ")")) {
            throw new DOMException("Invalid selector", "SyntaxError");
        }
        const relative = group.slice(contentStart, end - 1);
        predicates.push({ relative, negated: isNegated });
        offset = end + (isNegated ? 1 : 0);
    }
    return { base: base.trim() || "*", predicates };
}

function __matchesSelectorWithHas(element, selector) {
    selector = String(selector);
    if (!selector.includes(":has(")) return __callHost("matches", element, selector);
    return __splitSelectorGroups(selector).some(group => {
        const parsed = __selectorGroupWithHas(group);
        if (!__callHost("matches", element, parsed.base)) return false;
        return parsed.predicates.every(predicate => {
            const found = __callHost("querySelector", element, predicate.relative) !== null;
            return predicate.negated ? !found : found;
        });
    });
}

function __querySelectorAllWithHas(root, selector) {
    selector = String(selector);
    if (!selector.includes(":has(")) return __callHost("querySelectorAll", root, selector);
    const candidates = __callHost("querySelectorAll", root, "*");
    return candidates.filter(element => __matchesSelectorWithHas(element, selector));
}

function __querySelectorWithHas(root, selector) {
    const results = __querySelectorAllWithHas(root, selector);
    return results.length === 0 ? null : results[0];
}

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
