class CharacterData extends Node {
    get data() { return this.textContent; }
    set data(value) { this.textContent = String(value); }
    get length() { return this.data.length; }
    substringData(offset, count) {
        offset = Number(offset) >>> 0;
        count = Number(count) >>> 0;
        if (offset > this.length) throw new DOMException("offset is outside the data", "IndexSizeError");
        return this.data.substring(offset, Math.min(offset + count, this.length));
    }
    appendData(data) { this.replaceData(this.length,0,data); }
    insertData(offset, data) { this.replaceData(offset, 0, data); }
    deleteData(offset, count) { this.replaceData(offset, count, ""); }
    replaceData(offset, count, data) {
        offset = Number(offset) >>> 0;
        count = Number(count) >>> 0;
        if (offset > this.length) throw new DOMException("offset is outside the data", "IndexSizeError");
        data = String(data); count = Math.min(count,this.length-offset);
        __characterDataEdit = {offset,count,length:data.length};
        try { this.data = this.data.slice(0, offset) + data + this.data.slice(offset + count); }
        finally { __characterDataEdit = null; }
    }
    replaceWith(...nodes) { __replaceNode(this, nodes); }
    remove() { __removeNode(this); }
}

class Text extends CharacterData {
    splitText(offset) {
        offset = Number(offset) >>> 0;
        if (offset > this.length) throw new DOMException('Invalid offset','IndexSizeError');
        const node = this.ownerDocument.createTextNode(this.data.slice(offset));
        const parent = this.parentNode, index = parent ? parent.childNodes.indexOf(this) : -1;
        const moved = [];
        __rangeBoundaries((r,n,o) => {
            if (r[n] === this && r[o] > offset) moved.push(() => {r[n]=node;r[o]-=offset;});
            else if (parent && r[n] === parent && r[o] === index+1) moved.push(() => {r[o]++;});
        });
        if (parent) parent.insertBefore(node,this.nextSibling);
        for (const update of moved) update();
        this.deleteData(offset,this.length-offset);
        return node;
    }
    constructor(data = "") { return document.createTextNode(String(data)); }
    remove() { __removeNode(this); }
}
class Comment extends CharacterData {
    constructor(data = "") { return document.createComment(String(data)); }
}
class CDATASection extends Text {
    constructor() { throw new TypeError("Illegal constructor"); }
}
class ProcessingInstruction extends CharacterData {
    constructor() { throw new TypeError("Illegal constructor"); }
    get target() { return ""; }
}
class DocumentFragment extends Node {
    constructor() { return document.createDocumentFragment(); }
    get children() { return new HTMLCollection(() => [...this.childNodes].filter(node => node instanceof Element)); }
    get childElementCount() { return this.children.length; }
    get firstElementChild() { return this.children.item(0); }
    get lastElementChild() { return this.children.item(this.children.length - 1); }
    getElementById(id) {
        id = String(id);
        if (id === "") return null;
        return [...this.querySelectorAll("[id]")].find(element => element.id === id) ?? null;
    }
    querySelector(selector) { return __callHost("querySelector", this, String(selector)); }
    querySelectorAll(selector) { return new NodeList(__callHost("querySelectorAll", this, String(selector))); }
    append(...nodes) { __appendNodes(this, nodes); }
    prepend(...nodes) { __prependNodes(this, nodes); }
    replaceChildren(...nodes) { __replaceChildren(this, nodes); }
}
class WindowProperties extends EventTarget {
    constructor() {
        super();
        throw new TypeError("Illegal constructor");
    }
}
function Window() {
    throw new TypeError("Illegal constructor");
}
Window.prototype = Object.getPrototypeOf(globalThis);
Object.setPrototypeOf(Window.prototype, WindowProperties.prototype);
Object.defineProperty(Window.prototype, "constructor", {
    value: Window,
    writable: true,
    enumerable: false,
    configurable: true,
});
Object.defineProperties(Window.prototype, {
    frames: { get() { return this; }, enumerable: true, configurable: true },
    length: {
        get() { return document.getElementsByTagName("iframe").length; },
        enumerable: true,
        configurable: true,
    },
});

class Location {
    get href() { return __callHost("location", this, "href"); }
    set href(value) { this.assign(value); }
    get protocol() { return __callHost("location", this, "protocol"); }
    get host() { return __callHost("location", this, "host"); }
    get hostname() { return __callHost("location", this, "hostname"); }
    get port() { return __callHost("location", this, "port"); }
    get pathname() { return __callHost("location", this, "pathname"); }
    get search() { return __callHost("location", this, "search"); }
    get hash() { return __callHost("location", this, "hash"); }
    set hash(value) { const url = new URL(this.href); url.hash = String(value); this.assign(url.href); }
    get origin() { return __callHost("location", this, "origin"); }
    assign(url) {
        __navigateLocation(this, url, false);
    }
    replace(url) {
        __navigateLocation(this, url, true);
    }
    reload() { __callHost("locationNavigate", this, this.href); }
    toString() { return this.href; }
}

function __navigateLocation(receiver, value, replace) {
    const oldURL = receiver.href, next = new URL(__toUSVString(value), oldURL);
    const current = new URL(oldURL), oldHash = current.hash;
    current.hash = ''; const destination = new URL(next.href); destination.hash = '';
    if (current.href === destination.href && (next.hash || oldHash)) {
        if (next.href === oldURL) return;
        if (replace) history.replaceState(null, '', next.href); else history.pushState(null, '', next.href);
        setTimeout(() => {
            window.dispatchEvent(__markTrustedEvent(new PopStateEvent('popstate', {state:null})));
            window.dispatchEvent(__markTrustedEvent(new HashChangeEvent('hashchange', {oldURL, newURL:next.href})));
        }, 0);
    } else __callHost('locationNavigate', receiver, next.href);
}

class Navigator {
    constructor() { throw new TypeError("Illegal constructor"); }
    get userAgent() { return "Brimp/0.1"; }
    get platform() { return "MacIntel"; }
    get language() { return "en-US"; }
    get languages() { return ["en-US", "en"]; }
}

const __historyConstructorToken = {};
const __historyEntries = new WeakMap();
const __historyIndexes = new WeakMap();
const __historyScrollRestoration = new WeakMap();

function __historyData(history) {
    const entries = __historyEntries.get(history);
    if (entries === undefined) throw new TypeError("History method called on an incompatible receiver");
    return { entries, index: __historyIndexes.get(history) };
}


function __historyUrl(url) {
    if (url === undefined || url === null || url === "") return location.href;
    const resolved = new URL(__toUSVString(url), location.href);
    if (resolved.origin !== location.origin) {
        throw new DOMException("History state URL must be same-origin", "SecurityError");
    }
    return resolved.href;
}

class History {
    constructor(token) {
        if (token !== __historyConstructorToken) throw new TypeError("Illegal constructor");
        __historyEntries.set(this, [{ url: location.href, state: null }]);
        __historyIndexes.set(this, 0);
        __historyScrollRestoration.set(this, "auto");
    }
    get length() { return __historyData(this).entries.length; }
    get state() {
        const { entries, index } = __historyData(this);
        return entries[index].state;
    }
    get scrollRestoration() { __historyData(this); return __historyScrollRestoration.get(this); }
    set scrollRestoration(value) {
        __historyData(this);
        value = String(value);
        if (value === "auto" || value === "manual") __historyScrollRestoration.set(this, value);
    }
    pushState(data, unused, url = null) {
        if (arguments.length < 2) throw new TypeError("History.pushState requires data and title");
        String(unused);
        const state = structuredClone(data);
        const href = __callHost("historyUpdateUrl", this, __historyUrl(url));
        const { entries, index } = __historyData(this);
        entries.splice(index + 1);
        entries.push({ url: href, state });
        __historyIndexes.set(this, entries.length - 1);
    }
    replaceState(data, unused, url = null) {
        if (arguments.length < 2) throw new TypeError("History.replaceState requires data and title");
        String(unused);
        const state = structuredClone(data);
        const href = __callHost("historyUpdateUrl", this, __historyUrl(url));
        const { entries, index } = __historyData(this);
        entries[index] = { url: href, state };
    }
    go(delta = 0) {
        delta = Number(delta);
        if (!Number.isFinite(delta)) delta = 0;
        delta = Math.trunc(delta);
        if (delta === 0) return;
        const { entries, index } = __historyData(this);
        const destination = index + delta;
        if (destination < 0 || destination >= entries.length) return;
        setTimeout(() => {
            const oldURL = location.href;
            __historyIndexes.set(this, destination);
            const entry = entries[destination];
            const newURL = __callHost("historyUpdateUrl", this, entry.url);
            window.dispatchEvent(__markTrustedEvent(new PopStateEvent("popstate", { state: entry.state })));
            if (new URL(oldURL).hash !== new URL(newURL).hash) {
                window.dispatchEvent(__markTrustedEvent(new HashChangeEvent("hashchange", { oldURL, newURL })));
            }
        }, 0);
    }
    back() { this.go(-1); }
    forward() { this.go(1); }
}
