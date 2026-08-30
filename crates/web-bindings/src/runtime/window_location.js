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
    appendData(data) { this.data += String(data); }
    insertData(offset, data) { this.replaceData(offset, 0, data); }
    deleteData(offset, count) { this.replaceData(offset, count, ""); }
    replaceData(offset, count, data) {
        offset = Number(offset) >>> 0;
        count = Number(count) >>> 0;
        if (offset > this.length) throw new DOMException("offset is outside the data", "IndexSizeError");
        this.data = this.data.slice(0, offset) + String(data) + this.data.slice(offset + count);
    }
    replaceWith(...nodes) { __replaceNode(this, nodes); }
    remove() { __removeNode(this); }
}

class Text extends CharacterData {
    constructor(data = "") { return document.createTextNode(String(data)); }
    remove() { __removeNode(this); }
}
class Comment extends CharacterData {
    constructor(data = "") { return document.createComment(String(data)); }
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
    querySelector(selector) { return __querySelectorWithHas(this, selector); }
    querySelectorAll(selector) { return new NodeList(__querySelectorAllWithHas(this, selector)); }
    append(...nodes) { __appendNodes(this, nodes); }
    prepend(...nodes) { __prependNodes(this, nodes); }
    replaceChildren(...nodes) { __replaceChildren(this, nodes); }
}
class Window extends EventTarget {}
Object.defineProperty(Window, Symbol.hasInstance, {
    value(object) { return object === globalThis; },
});
Object.defineProperties(Window.prototype, {
    innerWidth: { get() { return __callHost("innerWidth", this); } },
    innerHeight: { get() { return __callHost("innerHeight", this); } },
    devicePixelRatio: { get() { return __callHost("devicePixelRatio", this); } },
});

class Location {
    get href() { return __callHost("location", this, "href"); }
    get protocol() { return __callHost("location", this, "protocol"); }
    get host() { return __callHost("location", this, "host"); }
    get hostname() { return __callHost("location", this, "hostname"); }
    get port() { return __callHost("location", this, "port"); }
    get pathname() { return __callHost("location", this, "pathname"); }
    get search() { return __callHost("location", this, "search"); }
    get hash() { return __callHost("location", this, "hash"); }
    get origin() { return __callHost("location", this, "origin"); }
    toString() { return this.href; }
}

class Navigator {
    get userAgent() { return "Brimp/0.1"; }
    get platform() { return "MacIntel"; }
    get language() { return "en-US"; }
    get languages() { return ["en-US", "en"]; }
}
