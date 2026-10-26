// Internal DOM operations must not dispatch through page-overridden methods.
const __childAppend = Function.prototype.call.bind(Node.prototype.appendChild);
const __childInsert = Function.prototype.call.bind(Node.prototype.insertBefore);
const __childReplace = Function.prototype.call.bind(Node.prototype.replaceChild);
const __childRemove = Function.prototype.call.bind(Node.prototype.removeChild);

function __removeNode(node) {
    const parent = __callHost('parentNode', node);
    if (parent) __childRemove(parent, node);
}

function __childNodeArguments(values) {
    // Web IDL conversion precedes the method steps, including the detached-node
    // early return. Template conversion rejects Symbols and uses the string hint.
    return values.map(value => value instanceof Node ? value : `${value}`);
}

function __childNodeReplacement(node, values) {
    const owner = node.ownerDocument;
    const nodes = values.map(value => typeof value === 'string' ? __callHost('createTextNode', owner, value) : value);
    if (nodes.length === 1) return nodes[0];
    const fragment = __callHost('createDocumentFragment', owner);
    for (const child of nodes) __childAppend(fragment, child);
    return fragment;
}

function __beforeNode(node, values) {
    const nodes = __childNodeArguments(values);
    const parent = node.parentNode;
    if (!parent) return;
    let previous = node.previousSibling;
    while (nodes.includes(previous)) previous = previous.previousSibling;
    const replacement = __childNodeReplacement(node, nodes);
    __childInsert(parent, replacement, previous ? previous.nextSibling : parent.firstChild);
}

function __afterNode(node, values) {
    const nodes = __childNodeArguments(values);
    const parent = node.parentNode;
    if (!parent) return;
    let reference = node.nextSibling;
    while (nodes.includes(reference)) reference = reference.nextSibling;
    __childInsert(parent, __childNodeReplacement(node, nodes), reference);
}

function __replaceNode(node, values) {
    const nodes = __childNodeArguments(values);
    const parent = node.parentNode;
    if (!parent) return;
    let reference = node.nextSibling;
    while (nodes.includes(reference)) reference = reference.nextSibling;
    const replacement = __childNodeReplacement(node, nodes);
    if (node.parentNode === parent) __childReplace(parent, replacement, node);
    else __childInsert(parent, replacement, reference);
}

class DocumentType extends Node {
    constructor() { throw new TypeError('Illegal constructor'); }
    get name() { return __callHost('nodeName', this); }
    get publicId() { return __callHost('doctypePublicId', this); }
    get systemId() { return __callHost('doctypeSystemId', this); }
    before(...nodes) { __beforeNode(this, nodes); }
    after(...nodes) { __afterNode(this, nodes); }
    replaceWith(...nodes) { __replaceNode(this, nodes); }
    remove() { __removeNode(this); }
}

const __documentWindows = new WeakMap();
const __documentDesignModes = new WeakMap();
function __documentWindow(owner) {
    if (owner === document) return window;
    const context = __documentWindows.get(owner);
    if (!context) return null;
    let root = context.frame, parent;
    while ((parent = __callHost('parentNode', root) ?? __shadowData.get(root)?.host) != null) root = parent;
    return root !== owner && __documentWindow(root) !== null ? context.window : null;
}
function __documentURL(owner) {
    return __callHost('documentURL', owner);
}
function __documentHtmlRoot(owner) {
    const root = __callHost('documentElement', owner);
    return root && __callHost('namespaceURI', root) === 'http://www.w3.org/1999/xhtml'
        && __callHost('localName', root) === 'html' ? root : null;
}
function __documentCollection(owner, name) {
    return __cachedCollection(owner, name, () => __createHTMLCollection(() => __callHost('documentCollection', owner, name)));
}

class Document extends Node {
    constructor() {
        const created = __callHost('createDocument', null);
        const prototype = new.target.prototype;
        Object.setPrototypeOf(created, prototype !== null && (typeof prototype === 'object' || typeof prototype === 'function') ? prototype : Document.prototype);
        return created;
    }
    get readyState() { return this === document ? __documentReadyState : 'complete'; }
    get designMode() { return __documentDesignModes.get(this) ?? 'off'; }
    set designMode(value) {
        const mode = __asciiLowercase(`${value}`);
        if (mode === 'on' || mode === 'off') __documentDesignModes.set(this, mode);
    }
    get rootElement() {
        const root = this.documentElement;
        return root?.namespaceURI === 'http://www.w3.org/2000/svg' && root.localName === 'svg' ? root : null;
    }
    get title() {
        const title = __callHost('documentTitleElement', this);
        if (!title) return '';
        let text = '';
        if (__callHost('namespaceURI', title) === 'http://www.w3.org/2000/svg') {
            text = __callHost('textContent', title);
        } else {
            for (const child of __callHost('childNodes', title)) {
                if (__callHost('nodeType', child) === 3) text += __callHost('textContent', child);
            }
        }
        return text.replace(/[\t\n\f\r ]+/g, ' ').replace(/^ | $/g, '');
    }
    set title(value) {
        value = `${value}`;
        const root = __callHost('documentElement', this);
        if (!root) return;
        const namespace = __callHost('namespaceURI', root);
        const svg = namespace === 'http://www.w3.org/2000/svg' && __callHost('localName', root) === 'svg';
        if (!svg && namespace !== 'http://www.w3.org/1999/xhtml') return;
        let title = __callHost('documentTitleElement', this);
        if (!title) {
            const parent = svg ? root : __callHost('childNodes', root).find(child =>
                __callHost('nodeType', child) === 1 && __callHost('namespaceURI', child) === namespace
                    && __callHost('localName', child) === 'head');
            if (!parent) return;
            title = __callHost('createElementNS', this, namespace, 'title');
            __callHost('insertBefore', parent, title, svg ? __callHost('firstChild', parent) : null);
        }
        __callHost('setTextContent', title, value);
    }
    get URL() { return __documentURL(this) || 'about:blank'; }
    get documentURI() { return __documentURL(this) || 'about:blank'; }
    get contentType() { return __callHost('documentContentType', this); }
    get compatMode() { return __callHost('compatMode', this); }
    get doctype() { return __callHost('doctype', this); }
    get location() { return this === document ? globalThis.location : null; }
    set location(value) { if (this === document) globalThis.location.href = value; }
    get defaultView() { return __documentWindow(this); }
    get currentScript() { return this === document ? __currentScript : null; }
    get domain() {
        if (this.__domain !== undefined) return this.__domain;
        return this.location?.hostname ?? "";
    }
    set domain(value) {
        value = String(value).toLowerCase().replace(/\.$/, "");
        const current = this.location?.hostname.toLowerCase().replace(/\.$/, "") ?? "";
        if (value === "" || (value !== current && !current.endsWith("." + value))) {
            throw new DOMException("The document domain cannot be set to this value", "SecurityError");
        }
        this.__domain = value;
    }
    get dir() {
        const root = __documentHtmlRoot(this);
        const value = __asciiLowercase(root ? __getContentAttribute(root, 'dir') ?? '' : '');
        return value === "ltr" || value === "rtl" || value === "auto" ? value : "";
    }
    set dir(value) {
        value = `${value}`;
        const root = __documentHtmlRoot(this);
        if (root) __setContentAttribute(root, 'dir', value);
    }
    get hidden() { return false; }
    get visibilityState() { return "visible"; }
    get characterSet() { return __callHost("documentEncoding", this); }
    get charset() { return __callHost("documentEncoding", this); }
    get inputEncoding() { return __callHost("documentEncoding", this); }
    get cookie() { return __callHost("cookie", this); }
    set cookie(value) { __callHost("setCookie", this, value); }
    get documentElement() { return __callHost("documentElement", this); }
    get head() { return __callHost("head", this); }
    get body() { return __callHost("body", this); }
    set body(value) {
        if (value != null && (!['object', 'function'].includes(typeof value)
            || !__callHost('isNativeNode', value) || __callHost('nodeType', value) !== 1
            || __callHost('namespaceURI', value) !== 'http://www.w3.org/1999/xhtml')) {
            throw new TypeError('Document body must be an HTMLElement');
        }
        if (value == null || !['body', 'frameset'].includes(__callHost('localName', value))) {
            throw new DOMException('Document body must be a body or frameset element', 'HierarchyRequestError');
        }
        const previous = __callHost('body', this);
        if (value === previous) return;
        const root = __callHost('documentElement', this);
        if (!root) throw new DOMException('Document has no root element', 'HierarchyRequestError');
        for (let parent = root; parent; parent = __callHost('parentNode', parent) ?? __shadowData.get(parent)?.host) {
            if (parent === value) throw new DOMException('Invalid tree insertion', 'HierarchyRequestError');
        }
        const owner = __callHost('ownerDocument', value);
        if (previous) __callHost('replaceChild', root, value, previous);
        else __callHost('appendChild', root, value);
        __disconnectCustomElementTree(value);
        if (owner !== this) for (const element of __customElementTree(value)) {
            if (__upgradedCustomElements.has(element) && typeof element.adoptedCallback === 'function') {
                element.adoptedCallback(owner, this);
            }
        }
        if (previous) __disconnectCustomElementTree(previous);
        __connectCustomElementTree(value);
    }
    get forms() { return __documentCollection(this, 'forms'); }
    get images() { return __documentCollection(this, 'images'); }
    get embeds() { return __documentCollection(this, 'embeds'); }
    get plugins() { return __documentCollection(this, 'embeds'); }
    get links() { return __documentCollection(this, 'links'); }
    get scripts() { return __documentCollection(this, 'scripts'); }
    get anchors() { return __documentCollection(this, 'anchors'); }
    get applets() { return __documentCollection(this, 'applets'); }
    get activeElement() {
        return __activeElement?.ownerDocument === this && __activeElement.isConnected
            ? __activeElement : this.body ?? this.documentElement;
    }
    get children() { return __elementChildren(this); }
    get childElementCount() { return __callHost('childElementCount', this); }
    get firstElementChild() { return __callHost('firstElementChild', this); }
    get lastElementChild() { return __callHost('lastElementChild', this); }
    get implementation() { return __domImplementation; }
    createElement(name) {
        name = String(name); __validateXmlName(name);
        return __upgradeCustomElement(__callHost('createElement', this, name), true);
    }
    createElementNS(namespace, qualifiedName) {
        const name = __validateQualifiedName(namespace, qualifiedName);
        return __callHost("createElementNS", this, name.namespaceURI, name.name);
    }
    createTextNode(text) { return __callHost("createTextNode", this, text); }
    createComment(data) { return __callHost("createComment", this, data); }
    createCDATASection(data) {
        // HTML rejects CDATA; the runtime also has no XML CDATA node backend.
        throw new DOMException('CDATA sections are not supported', 'NotSupportedError');
    }
    // Legacy callable placeholder until an XML node backend is in scope.
    createProcessingInstruction(target, data) {}
    createDocumentFragment() { return __callHost("createDocumentFragment", this); }
    adoptNode(node) {
        if (!(node instanceof Node)) throw new TypeError('Expected a Node');
        if (node instanceof Document || node instanceof ShadowRoot) throw new DOMException('Cannot adopt this node', 'NotSupportedError');
        const previous = node instanceof Attr ? node.ownerDocument : __callHost('ownerDocument', node);
        if (node instanceof Attr) {
            if (node.ownerElement) node.ownerElement.removeAttributeNode(node);
            node.__ownerDocument = this;
        } else {
            if (node.parentNode) node.parentNode.removeChild(node);
            __callHost('adoptNode', this, node);
            if (previous !== this) for (const e of __customElementTree(node)) {
                if (__upgradedCustomElements.has(e) && typeof e.adoptedCallback === 'function') e.adoptedCallback(previous, this);
            }
        }
        return node;
    }
    createAttribute(name) {
        name = __asciiLowercase(String(name)); __validateXmlName(name);
        return new Attr(__attrConstructionToken, null, {name,localName:name,prefix:null,namespaceURI:null,value:''}, this);
    }
    createAttributeNS(namespace, name) {
        const record = __validateQualifiedName(namespace, name);
        return new Attr(__attrConstructionToken, null, {...record, value:''}, this);
    }
    importNode(node, deep = false) {
        if (!(node instanceof Node)) throw new TypeError("node must be a Node");
        // Safari also rejects importing DocumentType, although adoption works.
        if (node.nodeType === Node.DOCUMENT_NODE || node.nodeType === Node.DOCUMENT_TYPE_NODE) {
            throw new DOMException("This node type cannot be imported", "NotSupportedError");
        }
        if (node instanceof Attr) return new Attr(__attrConstructionToken, null, {...node.__record, value:node.value}, this);
        return __callHost("importNode", this, node, Boolean(deep));
    }
    hasFocus() { return true; }
    createTreeWalker(root, whatToShow = NodeFilter.SHOW_ALL, filter = null) {
        if (!(root instanceof Node)) throw new TypeError("root must be a Node");
        return new TreeWalker(root, whatToShow, filter);
    }
    createNodeIterator(root, whatToShow = NodeFilter.SHOW_ALL, filter = null) {
        if (!(root instanceof Node)) throw new TypeError("root must be a Node");
        return new NodeIterator(root, whatToShow, filter, __nodeIteratorToken);
    }
    createRange() { return new Range(this); }
    getSelection() { return __selection; }
    append(...nodes) { __appendNodes(this, nodes); }
    prepend(...nodes) { __prependNodes(this, nodes); }
    replaceChildren(...nodes) { __replaceChildren(this, nodes); }
    createEvent(interfaceName) {
        const create = constructor => {
            const event = new constructor('');
            __uninitializedEvents.add(event);
            return event;
        };
        switch (String(interfaceName).toLowerCase()) {
            case "event":
            case "events":
            case "htmlevents":
            case "svgevents": return create(Event);
            case "customevent": return create(CustomEvent);
            case "beforeunloadevent": return __createBeforeUnloadEvent();
            case "uievent":
            case "uievents": return create(UIEvent);
            case "textevent": return __createTextEvent();
            case "focusevent": return create(FocusEvent);
            case "mouseevent":
            case "mouseevents": return create(MouseEvent);
            case "keyboardevent": return create(KeyboardEvent);
            default: throw new DOMException("The event interface is not supported", "NotSupportedError");
        }
    }
    getElementById(id) { return __callHost("getElementById", this, id); }
    getElementsByTagName(name) {
        return __createHTMLCollection(() => __callHost("getElementsByTagName", this, name));
    }
    getElementsByClassName(names) {
        return __createHTMLCollection(() => __callHost("getElementsByClassName", this, names));
    }
    getElementsByName(name) {
        return __createNodeList(() => __callHost("getElementsByName", this, name));
    }
    querySelector(selector) { return __selectorCall("querySelector", this, __selectorString(selector, arguments.length)); }
    querySelectorAll(selector) { return __createNodeList(__selectorCall("querySelectorAll", this, __selectorString(selector, arguments.length))); }
}


class DOMParser {
    parseFromString(input, type) {
        if (arguments.length < 2) throw new TypeError("two arguments are required");
        input = String(input);
        type = String(type);
        if (type !== "text/html") {
            throw new TypeError("unsupported DOMParser type");
        }
        const parsed = __callHost("domParserParse", this, input, type, __safariDOMParserStandardsMode(input));
        return parsed;
    }
}

function __boundaryLength(node) {
    if (!(node instanceof Node)) throw new TypeError("boundary container must be a Node");
    return node instanceof CharacterData ? node.length : node.childNodes.length;
}

function __validateBoundary(node, offset) {
    if (!(node instanceof Node)) throw new TypeError("boundary container must be a Node");
    if(node.nodeType===10) throw new DOMException('DocumentType cannot be a boundary','InvalidNodeTypeError');
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
        return aOffset <= Array.prototype.indexOf.call(aNode.childNodes, child) ? -1 : 1;
    }

    child = aNode;
    while (child.parentNode && child.parentNode !== bNode) child = child.parentNode;
    if (child.parentNode === bNode) {
        return bOffset <= Array.prototype.indexOf.call(bNode.childNodes, child) ? 1 : -1;
    }

    return aNode.compareDocumentPosition(bNode) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
}

// Weak registrations let ranges remain live without retaining abandoned ranges.
const __liveRanges = new Set();
function __eachRange(callback) {
    for (const reference of __liveRanges) {
        const range = reference.deref();
        if (range) callback(range); else __liveRanges.delete(reference);
    }
}
function __rangeBoundaries(callback) {
    __eachRange(range => { for (const side of ['start','end']) callback(range, `__${side}Container`, `__${side}Offset`); });
}
let __characterDataEdit = null;
function __rangeMutationBefore(operation, args) {
    if (!__liveRanges.size) return () => {};
    const [target, child, reference] = args;
    const updates = [];
    const remove = (node, parent, index) => updates.push(() => __rangeBoundaries((r,n,o) => {
        if (node.contains(r[n])) { r[n] = parent; r[o] = index; }
        else if (r[n] === parent && r[o] > index) r[o]--;
    }));
    if (operation === 'replaceChild') {
        const incoming = child.nodeType === 11 ? Array.from(child.childNodes) : [child];
        let index = Array.prototype.indexOf.call(target.childNodes, reference);
        for (const node of incoming) if (node.parentNode) {
            const parent = node.parentNode, oldIndex = Array.prototype.indexOf.call(parent.childNodes, node);
            remove(node,parent,child.nodeType === 11 ? 0 : oldIndex);
            if(parent === target && oldIndex < index) index--;
        }
        if (child !== reference) remove(reference,target,index);
        updates.push(() => __rangeBoundaries((r,n,o) => { if(r[n]===target && r[o]>index)r[o]+=incoming.length; }));
    } else if (['appendChild','insertBefore','removeChild'].includes(operation)) {
        const children = child.nodeType === 11 ? Array.from(child.childNodes) : [child];
        for (const node of children) {
            const parent = __callHost('parentNode', node);
            if (parent) remove(node,parent,child.nodeType === 11 ? 0 : Array.prototype.indexOf.call(parent.childNodes, node));
        }
        if (operation !== 'removeChild') {
            let index = operation === 'appendChild' || reference == null ? target.childNodes.length : Array.prototype.indexOf.call(target.childNodes, reference);
            if (child.parentNode === target && Array.prototype.indexOf.call(target.childNodes, child) < index) index--;
            // insertBefore(node, node) removes then reinserts at the following sibling.
            updates.push(() => __rangeBoundaries((r,n,o) => { if (r[n] === target && r[o] > index) r[o] += children.length; }));
        }
    } else if (operation === 'setTextContent' && [3,4,7,8].includes(__callHost('nodeType', target))) {
        const edit = __characterDataEdit?.target === target ? __characterDataEdit
            : {offset:0,count:__callHost('textContent', target).length,length:String(child ?? '').length};
        updates.push(() => __rangeBoundaries((r,n,o) => {
            if (r[n] !== target) return;
            if (r[o] > edit.offset && r[o] <= edit.offset+edit.count) r[o] = edit.offset;
            else if (r[o] > edit.offset+edit.count) r[o] += edit.length-edit.count;
        }));
    } else if (operation === 'setTextContent' || operation === 'setInnerHTML') {
        const parent = target instanceof HTMLTemplateElement ? target.content : target;
        // replace-all removes children from the front; every old child boundary collapses to zero.
        for (const node of parent.childNodes) remove(node,parent,0);
    }
    return () => { for (const update of updates) update(); };
}
function __rangeContents(range, mode) {
    const doc = range.__document, fragment = doc.createDocumentFragment();
    if (range.collapsed) return fragment;
    const start = range.startContainer, startOffset = range.startOffset, end = range.endContainer, endOffset = range.endOffset;
    let collapseNode = start, collapseOffset = startOffset;
    if (!start.contains(end)) {
        let child = start;
        while (child.parentNode && !child.parentNode.contains(end)) child = child.parentNode;
        collapseNode = child.parentNode;
        collapseOffset = Array.prototype.indexOf.call(collapseNode.childNodes, child)+1;
    }
    const plan = node => {
        if (node instanceof CharacterData) {
            const first = node === start ? startOffset : 0, last = node === end ? endOffset : node.length;
            return {node, first, last, partial:node === start || node === end};
        }
        const children = [];
        for (const child of node.childNodes) {
            const index = Array.prototype.indexOf.call(node.childNodes, child);
            if (__boundaryOrder(node,index+1,start,startOffset)<=0 || __boundaryOrder(node,index,end,endOffset)>=0) continue;
            if (child.nodeType === 10 && mode !== 'delete') throw new DOMException('DocumentType cannot be cloned into a fragment','HierarchyRequestError');
            const partial = child.contains(start) || child.contains(end);
            children.push(partial ? plan(child) : {node:child,partial:false});
        }
        return {node, children, partial:true};
    };
    const tree = plan(range.commonAncestorContainer);
    // Track the collapse point through the removals performed below.
    const collapse = range.cloneRange(); collapse.setStart(collapseNode,collapseOffset); collapse.collapse(true);
    const apply = (item, output) => {
        const node = item.node;
        if (!item.partial) {
            if (mode === 'clone') output.appendChild(node.cloneNode(true));
            else if (mode === 'extract') output.appendChild(node);
            else node.parentNode.removeChild(node);
        } else if (node instanceof CharacterData) {
            if (mode !== 'delete') { const copy=node.cloneNode(false);copy.data=node.data.slice(item.first,item.last);output.appendChild(copy); }
            if (mode !== 'clone') node.deleteData(item.first,item.last-item.first);
        } else {
            const copy = mode === 'delete' ? null : node.cloneNode(false);
            for (const child of item.children) apply(child,copy);
            if (copy) output.appendChild(copy);
        }
    };
    if (tree.node instanceof CharacterData) apply(tree,fragment);
    else for (const child of tree.children) apply(child,fragment);
    if (mode !== 'clone') {range.setStart(collapse.startContainer,collapse.startOffset);range.collapse(true);}
    return fragment;
}

class Range {
    constructor(document = globalThis.document) {
        __liveRanges.add(new WeakRef(this));
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
    setStartBefore(node) { this.setStart(node.parentNode, Array.prototype.indexOf.call(node.parentNode.childNodes, node)); }
    setStartAfter(node) { this.setStart(node.parentNode, Array.prototype.indexOf.call(node.parentNode.childNodes, node) + 1); }
    setEndBefore(node) { this.setEnd(node.parentNode, Array.prototype.indexOf.call(node.parentNode.childNodes, node)); }
    setEndAfter(node) { this.setEnd(node.parentNode, Array.prototype.indexOf.call(node.parentNode.childNodes, node) + 1); }
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
        const index = Array.prototype.indexOf.call(parent.childNodes, node);
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
    cloneContents() { return __rangeContents(this, 'clone'); }
    extractContents() { return __rangeContents(this, 'extract'); }
    deleteContents() { __rangeContents(this, 'delete'); }
    insertNode(node) {
        if (!(node instanceof Node)) throw new TypeError('Expected a Node');
        const start = this.startContainer;
        if (start.nodeType === 8 || start.nodeType === 7 || (!start.parentNode && start.nodeType === 3) || start === node)
            throw new DOMException('Invalid insertion boundary', 'HierarchyRequestError');
        const collapsed = this.collapsed;
        let reference = start.nodeType === 3 ? start : start.childNodes[this.startOffset] ?? null;
        const parent = reference ? reference.parentNode : start;
        if (node.contains(parent)) throw new DOMException('Invalid insertion', 'HierarchyRequestError');
        if (start.nodeType === 3) reference = start.splitText(this.startOffset);
        if (reference === node) reference = reference.nextSibling;
        if (node.parentNode) node.parentNode.removeChild(node);
        const offset = reference ? Array.prototype.indexOf.call(parent.childNodes, reference) : parent.childNodes.length;
        const count = node.nodeType === 11 ? node.childNodes.length : 1;
        parent.insertBefore(node, reference);
        if (collapsed) this.setEnd(parent, offset + count);
    }
    surroundContents(parent) {
        if (!(parent instanceof Node)) throw new TypeError('Expected a Node');
        if ([9,10,11].includes(parent.nodeType)) throw new DOMException('Invalid parent', 'InvalidNodeTypeError');
        const visit = node => {
            if (node.nodeType !== 3 && node.contains(this.startContainer) !== node.contains(this.endContainer))
                throw new DOMException('Partially selected non-Text node', 'InvalidStateError');
            for (const child of node.childNodes) visit(child);
        };
        for (const child of this.commonAncestorContainer.childNodes) visit(child);
        const content = this.extractContents();
        parent.textContent = '';
        this.insertNode(parent);
        parent.appendChild(content);
        this.selectNode(parent);
    }
    createContextualFragment(markup) {
        let context = this.startContainer;
        if (!(context instanceof Element)) context = context.parentElement;
        if (!context || (context.namespaceURI === 'http://www.w3.org/1999/xhtml' && context.localName === 'html'))
            context = this.__document.createElement('body');
        else context = context.cloneNode(false);
        context.innerHTML = String(markup);
        const fragment = this.__document.createDocumentFragment();
        const source = context instanceof HTMLTemplateElement ? context.content : context;
        while (source.firstChild) fragment.appendChild(source.firstChild);
        return fragment;
    }
    comparePoint(node, offset) {
        offset = __validateBoundary(node, offset);
        if (__nodeRoot(node) !== __nodeRoot(this.startContainer)) throw new DOMException('Different roots', 'WrongDocumentError');
        if (__boundaryOrder(node,offset,this.startContainer,this.startOffset)<0) return -1;
        if (__boundaryOrder(node,offset,this.endContainer,this.endOffset)>0) return 1;
        return 0;
    }
    isPointInRange(node, offset) {
        offset = __validateBoundary(node,offset);
        return __nodeRoot(node) === __nodeRoot(this.startContainer) && this.comparePoint(node,offset) === 0;
    }
    intersectsNode(node) {
        if (!(node instanceof Node)) throw new TypeError('Expected a Node');
        if (__nodeRoot(node) !== __nodeRoot(this.startContainer)) return false;
        if (!node.parentNode) return true;
        const parent = node.parentNode, index = Array.prototype.indexOf.call(parent.childNodes, node);
        return __boundaryOrder(parent,index+1,this.startContainer,this.startOffset)>0 && __boundaryOrder(parent,index,this.endContainer,this.endOffset)<0;
    }
    toString() {
        let text = '';
        const visit = node => {
            if (node.nodeType === 3 || node.nodeType === 4) {
                if (__boundaryOrder(node,node.length,this.startContainer,this.startOffset)>0 && __boundaryOrder(node,0,this.endContainer,this.endOffset)<0)
                    text += node.data.slice(node === this.startContainer ? this.startOffset : 0, node === this.endContainer ? this.endOffset : node.length);
            } else for (const child of node.childNodes) visit(child);
        };
        if (!this.collapsed) visit(this.commonAncestorContainer);
        return text;
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
        const index = Array.prototype.indexOf.call(parent.childNodes, node);
        const start = __boundaryOrder(this.__range.startContainer, this.__range.startOffset, parent, index);
        const end = __boundaryOrder(this.__range.endContainer, this.__range.endOffset, parent, index + 1);
        return allowPartialContainment ? start < 0 && end > 0 : start <= 0 && end >= 0;
    }
    deleteFromDocument() {}
    toString() { return this.__range ? this.__range.toString() : ""; }
}
const __selection = new Selection();

function __liveCollectionItems(read) {
    let version = -1, items;
    return () => {
        if (version !== __domVersion) { items = read(); version = __domVersion; }
        return items;
    };
}
const __htmlCollectionData = new WeakMap();
function __htmlCollectionState(collection) {
    const state = __htmlCollectionData.get(collection);
    if (!state) throw new TypeError('Illegal invocation');
    return state;
}
function __htmlCollectionNamedItem(state, name) {
    if (name === '') return null;
    const items = state.items();
    return items.find(element => __getContentAttribute(element, 'id') === name)
        ?? items.find(element => __callHost('namespaceURI', element) === 'http://www.w3.org/1999/xhtml' && __getContentAttribute(element, 'name') === name)
        ?? null;
}
function __createHTMLCollection(read, prototype = HTMLCollection.prototype) {
    const target = Object.create(prototype), state = {items: __liveCollectionItems(read)};
    const indexed = key => typeof key === 'string' && /^(0|[1-9][0-9]*)$/.test(key) && Number(key) < 4294967295;
    const named = key => typeof key === 'string' && !indexed(key) && !Reflect.has(target, key)
        ? __htmlCollectionNamedItem(state, key) : null;
    const protectedName = key => typeof key === 'string' && !indexed(key) && !Object.hasOwn(target, key)
        && __htmlCollectionNamedItem(state, key) !== null;
    const descriptor = key => {
        const index = indexed(key), value = index ? state.items()[Number(key)] : named(key);
        return value == null ? undefined : {value, writable:false, enumerable:index, configurable:true};
    };
    const collection = new Proxy(target, {
        get(target, key, receiver) {
            return descriptor(key)?.value ?? Reflect.get(target, key, receiver);
        },
        has(target, key) { return descriptor(key) !== undefined || Reflect.has(target, key); },
        getOwnPropertyDescriptor(target, key) {
            return descriptor(key) ?? Reflect.getOwnPropertyDescriptor(target, key);
        },
        ownKeys(target) {
            const nodes = state.items(), keys = new Set(nodes.map((_, index) => String(index)));
            for (const node of nodes) {
                for (const name of [__getContentAttribute(node, 'id'), __callHost('namespaceURI', node) === 'http://www.w3.org/1999/xhtml' ? __getContentAttribute(node, 'name') : null]) {
                    if (name) keys.add(name);
                }
            }
            for (const key of Reflect.ownKeys(target)) keys.add(key);
            return [...keys];
        },
        set(target, key, value, receiver) {
            if (receiver === collection ? indexed(key) || named(key) !== null
                : indexed(key) && Number(key) < state.items().length) return false;
            return Reflect.set(target, key, value, receiver);
        },
        defineProperty(target, key, property) {
            return !indexed(key) && !protectedName(key) && Reflect.defineProperty(target, key, property);
        },
        deleteProperty(target, key) {
            return !(indexed(key) && Number(key) < state.items().length) && !protectedName(key) && Reflect.deleteProperty(target, key);
        },
        // Native Safari collections can gain indexed keys after preventExtensions.
        // JS Proxy invariants cannot represent that, so reject it explicitly.
        preventExtensions() { return false; },
    });
    __htmlCollectionData.set(collection, state);
    return collection;
}
class HTMLCollection {
    constructor() { throw new TypeError('Illegal constructor'); }
    get length() { return __htmlCollectionState(this).items().length; }
    item(index) {
        const state = __htmlCollectionState(this);
        if (arguments.length < 1) throw new TypeError('Not enough arguments');
        index = +index >>> 0;
        return state.items()[index] ?? null;
    }
    namedItem(name) {
        const state = __htmlCollectionState(this);
        if (arguments.length < 1) throw new TypeError('Not enough arguments');
        return __htmlCollectionNamedItem(state, `${name}`);
    }
}
Object.defineProperty(HTMLCollection.prototype, Symbol.iterator, {
    value: Array.prototype.values, writable: true, configurable: true,
});
Object.defineProperty(HTMLCollection.prototype, Symbol.toStringTag, {
    value: 'HTMLCollection', configurable: true,
});

class HTMLFormControlsCollection extends HTMLCollection {}

const __nodeListData = new WeakMap();
function __nodeListItems(list) {
    const read = __nodeListData.get(list);
    if (!read) throw new TypeError('Illegal invocation');
    return read();
}
function __createNodeList(items = []) {
    const snapshot = typeof items === 'function' ? null : Array.from(items);
    const read = snapshot ? () => snapshot : __liveCollectionItems(items);
    const indexed = key => typeof key === 'string' && /^(0|[1-9][0-9]*)$/.test(key) && Number(key) < 4294967295;
    const descriptor = key => indexed(key) && Number(key) < read().length
        ? {value: read()[Number(key)], configurable:true, enumerable:true, writable:false} : undefined;
    const list = new Proxy(Object.create(NodeList.prototype), {
        get(target, key, receiver) { return descriptor(key)?.value ?? Reflect.get(target, key, receiver); },
        has(target, key) { return descriptor(key) !== undefined || Reflect.has(target, key); },
        getOwnPropertyDescriptor(target, key) { return descriptor(key) ?? Reflect.getOwnPropertyDescriptor(target, key); },
        ownKeys(target) { return [...read().keys()].map(String).concat(Reflect.ownKeys(target)); },
        set(target, key, value, receiver) {
            if (indexed(key) && (receiver === list || Number(key) < read().length)) return false;
            return Reflect.set(target, key, value, receiver);
        },
        defineProperty(target, key, property) { return !indexed(key) && Reflect.defineProperty(target, key, property); },
        deleteProperty(target, key) { return descriptor(key) === undefined && Reflect.deleteProperty(target, key); },
        // Like the other live DOM collections, these proxies cannot become
        // non-extensible while continuing to expose new indexed properties.
        preventExtensions() { return false; },
    });
    __nodeListData.set(list, read);
    return list;
}
class NodeList {
    constructor() { throw new TypeError('Illegal constructor'); }
    get length() { return __nodeListItems(this).length; }
    item(index) {
        if (!__nodeListData.has(this)) throw new TypeError('Illegal invocation');
        if (arguments.length < 1) throw new TypeError('Not enough arguments');
        index = +index >>> 0;
        return __nodeListItems(this)[index] ?? null;
    }
}
for (const name of ['entries', 'keys', 'values', 'forEach']) Object.defineProperty(NodeList.prototype, name, {
    value: Array.prototype[name], writable: true, enumerable: true, configurable: true,
});
Object.defineProperty(NodeList.prototype, Symbol.iterator, {
    value: Array.prototype.values, writable: true, configurable: true,
});
Object.defineProperty(NodeList.prototype, Symbol.toStringTag, {value: 'NodeList', configurable: true});

const __attrConstructionToken = {};
const __attributeNodes = new WeakSet();

class Attr extends Node {
    constructor(token, element, record, owner = element?.ownerDocument ?? document) {
        if (token !== __attrConstructionToken) throw new TypeError("Illegal constructor");
        super(__nodeConstructionToken);
        __attributeNodes.add(this);
        this.__element = element;
        this.__record = record;
        this.__ownerDocument = owner;
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
        value = `${value}`;
        const record = this.__currentRecord() ?? this.__record;
        if (this.__element !== null) {
            const element = this.__element;
            __callHost('setAttributeNS', element, record.namespaceURI ?? '', record.name, value);
            __notifyAttributeChange(element, record.localName, record.value, value, false, record.namespaceURI);
        } else this.__record.value = value;
    }
    get ownerElement() {
        this.__currentRecord();
        return this.__element;
    }
    get specified() { return true; }
    get nodeType() { return Node.ATTRIBUTE_NODE; }
    get nodeName() { return this.name; }
    get ownerDocument() { return this.__element?.ownerDocument ?? this.__ownerDocument; }
    get parentNode() { return null; }
    get parentElement() { return null; }
    get childNodes() { return __cachedCollection(this, "childNodes", () => __createNodeList()); }
    get firstChild() { return null; }
    get lastChild() { return null; }
    get previousSibling() { return null; }
    get nextSibling() { return null; }
    hasChildNodes() { return false; }
}

function __attributeRecords(element) {
    return __callHost("elementAttributes", element).map(([name, localName, namespaceURI, prefix, value]) =>
        ({name, localName, namespaceURI, prefix, value}));
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
        name = String(name);
        if (this.__element.namespaceURI === __htmlNamespace) name = __asciiLowercase(name);
        const record = __attributeRecords(this.__element).find(item => item.name === name);
        return record === undefined ? null : this.__attr(record);
    }
    removeNamedItem(name) {
        const attr = this.getNamedItem(name);
        if (attr === null) throw new DOMException("attribute was not found", "NotFoundError");
        return this.__element.removeAttributeNode(attr);
    }
    get [Symbol.toStringTag]() { return "NamedNodeMap"; }
    [Symbol.iterator]() {
        return __attributeRecords(this.__element).map(record => this.__attr(record))[Symbol.iterator]();
    }
}
