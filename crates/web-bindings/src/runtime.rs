use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use blitz_dom::{LocalName, NodeData, QualName, ns};
use browser_dom::{BrowserDocument, NodeId};
use jsc::{
    JsException, JsObjectIdentity, JsRuntime, NativeCall, NativeError, NativeValue,
    PromiseSettlement, ProtectedJsObject,
};

use crate::WrapperCache;

const CLASS_DEFINITIONS: &str = r#"
const __eventListeners = new WeakMap();

class DOMException extends Error {
    constructor(message = "", name = "Error") {
        super(String(message));
        this.name = String(name);
    }
    get code() {
        return DOMException.__codes[this.name] || 0;
    }
}
DOMException.__codes = {
    IndexSizeError: 1,
    HierarchyRequestError: 3,
    WrongDocumentError: 4,
    InvalidCharacterError: 5,
    NoModificationAllowedError: 7,
    NotFoundError: 8,
    NotSupportedError: 9,
    InUseAttributeError: 10,
    InvalidStateError: 11,
    SyntaxError: 12,
    InvalidModificationError: 13,
    NamespaceError: 14,
    InvalidAccessError: 15,
    TypeMismatchError: 17,
    SecurityError: 18,
    NetworkError: 19,
    AbortError: 20,
    URLMismatchError: 21,
    QuotaExceededError: 22,
    TimeoutError: 23,
    InvalidNodeTypeError: 24,
    DataCloneError: 25,
};
for (const [name, code] of Object.entries(DOMException.__codes)) {
    const constant = name.replace(/Error$/, "").replace(/([a-z])([A-Z])/g, "$1_$2").toUpperCase() + "_ERR";
    Object.defineProperty(DOMException, constant, { value: code, enumerable: true });
    Object.defineProperty(DOMException.prototype, constant, { value: code, enumerable: true });
}

class Event {
    constructor(type, options = {}) {
        if (arguments.length === 0) throw new TypeError("Event type is required");
        this.type = String(type);
        this.bubbles = Boolean(options.bubbles);
        this.cancelable = Boolean(options.cancelable);
        this.target = null;
        this.srcElement = null;
        this.currentTarget = null;
        this.eventPhase = 0;
        this.defaultPrevented = false;
        this.composed = Boolean(options.composed);
        this.isTrusted = false;
        this.timeStamp = Date.now();
        this.__stopped = false;
        this.__immediateStopped = false;
        this.__dispatching = false;
        this.__inPassiveListener = false;
    }
    preventDefault() {
        if (this.cancelable && !this.__inPassiveListener) this.defaultPrevented = true;
    }
    stopPropagation() { this.__stopped = true; }
    stopImmediatePropagation() {
        this.__immediateStopped = true;
        this.__stopped = true;
    }
    composedPath() { return this.__path ? this.__path.slice() : []; }
    initEvent(type, bubbles = false, cancelable = false) {
        if (arguments.length === 0) throw new TypeError("Event type is required");
        if (this.__dispatching) return;
        this.type = String(type);
        this.bubbles = Boolean(bubbles);
        this.cancelable = Boolean(cancelable);
        this.defaultPrevented = false;
        this.__stopped = false;
        this.__immediateStopped = false;
    }
    get cancelBubble() { return this.__stopped; }
    set cancelBubble(value) { if (value) this.stopPropagation(); }
    get returnValue() { return !this.defaultPrevented; }
    set returnValue(value) { if (!value) this.preventDefault(); }
}
Event.NONE = 0;
Event.CAPTURING_PHASE = 1;
Event.AT_TARGET = 2;
Event.BUBBLING_PHASE = 3;

class CustomEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        this.detail = options.detail === undefined ? null : options.detail;
    }
    initCustomEvent(type, bubbles = false, cancelable = false, detail = null) {
        if (arguments.length === 0) throw new TypeError("Event type is required");
        if (this.__dispatching) return;
        this.initEvent(type, bubbles, cancelable);
        this.detail = detail;
    }
}

function __listenerCapture(options) {
    return typeof options === "boolean" ? options : Boolean(options && options.capture);
}

function __invokeListeners(target, event, capture, phase) {
    const listeners = (__eventListeners.get(target) || []).slice();
    event.currentTarget = target;
    event.eventPhase = phase;
    event.__immediateStopped = false;
    for (const listener of listeners) {
        if (listener.type !== event.type || listener.capture !== capture) continue;
        if (listener.once) target.removeEventListener(event.type, listener.callback, listener.capture);
        event.__inPassiveListener = listener.passive;
        try {
            if (typeof listener.callback === "function") listener.callback.call(target, event);
            else listener.callback.handleEvent(event);
        } finally {
            event.__inPassiveListener = false;
        }
        if (event.__immediateStopped) break;
    }
}

class EventTarget {
    addEventListener(type, callback, options = false) {
        if (callback == null) return;
        const capture = __listenerCapture(options);
        const signal = options && typeof options === "object" ? options.signal : undefined;
        if (signal && signal.aborted) return;
        const listeners = __eventListeners.get(this) || [];
        if (!listeners.some(item => item.type === String(type) && item.callback === callback && item.capture === capture)) {
            const listener = {
                type: String(type), callback, capture,
                once: Boolean(options && typeof options === "object" && options.once),
                passive: Boolean(options && typeof options === "object" && options.passive),
            };
            listeners.push(listener);
            __eventListeners.set(this, listeners);
            if (signal) {
                signal.addEventListener("abort", () => {
                    this.removeEventListener(type, callback, capture);
                }, { once: true });
            }
        }
    }
    removeEventListener(type, callback, options = false) {
        const capture = __listenerCapture(options);
        const listeners = __eventListeners.get(this);
        if (!listeners) return;
        const index = listeners.findIndex(item => item.type === String(type) && item.callback === callback && item.capture === capture);
        if (index !== -1) listeners.splice(index, 1);
    }
    dispatchEvent(event) {
        if (!(event instanceof Event)) throw new TypeError("argument must be an Event");
        if (event.__dispatching) throw new Error("event is already being dispatched");
        event.__dispatching = true;
        event.target = this;
        const path = [];
        let ancestor = this instanceof Node ? this.parentNode : null;
        while (ancestor) { path.push(ancestor); ancestor = ancestor.parentNode; }
        if (this instanceof Node && path[path.length - 1] !== window) path.push(window);
        event.__path = [this, ...path];
        try {
            for (let i = path.length - 1; i >= 0 && !event.__stopped; i--) {
                __invokeListeners(path[i], event, true, Event.CAPTURING_PHASE);
            }
            if (!event.__stopped) {
                __invokeListeners(this, event, true, Event.AT_TARGET);
                if (!event.__immediateStopped) __invokeListeners(this, event, false, Event.AT_TARGET);
            }
            if (event.bubbles && !event.__stopped) {
                for (const target of path) {
                    __invokeListeners(target, event, false, Event.BUBBLING_PHASE);
                    if (event.__stopped) break;
                }
            }
            if (!event.__stopped) {
                const handler = this[`on${event.type}`];
                if (typeof handler === "function") handler.call(this, event);
            }
            return !event.defaultPrevented;
        } finally {
            event.currentTarget = null;
            event.eventPhase = Event.NONE;
            event.__dispatching = false;
            event.__path = [];
        }
    }
}

class AbortSignal extends EventTarget {
    constructor() {
        super();
        this.aborted = false;
        this.reason = undefined;
        this.onabort = null;
        this.__dependents = [];
    }
    throwIfAborted() {
        if (this.aborted) throw this.reason;
    }
    static abort(reason = undefined) {
        const signal = new AbortSignal();
        signal.__abort(reason);
        return signal;
    }
    static timeout(milliseconds) {
        milliseconds = Number(milliseconds);
        if (!Number.isFinite(milliseconds) || milliseconds < 0 || milliseconds > 0xffffffff) {
            throw new TypeError("timeout must be an unsigned long long");
        }
        const signal = new AbortSignal();
        setTimeout(() => signal.__abort(new DOMException("The operation timed out", "TimeoutError")), milliseconds);
        return signal;
    }
    static any(signals) {
        const inputs = Array.from(signals);
        for (const signal of inputs) {
            if (!(signal instanceof AbortSignal)) throw new TypeError("value is not an AbortSignal");
        }
        const result = new AbortSignal();
        const alreadyAborted = inputs.find(signal => signal.aborted);
        if (alreadyAborted) {
            result.__abort(alreadyAborted.reason);
            return result;
        }
        for (const signal of new Set(inputs)) signal.__dependents.push(result);
        return result;
    }
    __abort(reason = undefined) {
        if (this.aborted) return;
        const abortReason = reason === undefined
            ? new DOMException("The operation was aborted", "AbortError")
            : reason;
        const queue = [this];
        const aborted = [];
        while (queue.length) {
            const signal = queue.shift();
            if (signal.aborted) continue;
            signal.aborted = true;
            signal.reason = abortReason;
            aborted.push(signal);
            queue.push(...signal.__dependents);
        }
        for (const signal of aborted) {
            const event = new Event("abort");
            event.isTrusted = true;
            signal.dispatchEvent(event);
        }
    }
}

class AbortController {
    constructor() { this.signal = new AbortSignal(); }
    abort(reason = undefined) { this.signal.__abort(reason); }
}

class Node extends EventTarget {
    get nodeType() { return __brimp("nodeType", this); }
    get nodeName() { return __brimp("nodeName", this); }
    get parentNode() { return __brimp("parentNode", this); }
    get parentElement() {
        const parent = this.parentNode;
        return parent instanceof Element ? parent : null;
    }
    get firstChild() { return __brimp("firstChild", this); }
    get lastChild() { return __brimp("lastChild", this); }
    get childNodes() { return __brimp("childNodes", this); }
    get textContent() { return __brimp("textContent", this); }
    set textContent(value) { __brimp("setTextContent", this, value); }
    appendChild(child) { return __brimp("appendChild", this, child); }
    removeChild(child) { return __brimp("removeChild", this, child); }
    insertBefore(child, reference) { return __brimp("insertBefore", this, child, reference); }
}

class DOMImplementation {
    hasFeature() { return true; }
}
const __domImplementation = new DOMImplementation();

class Document extends Node {
    get title() { return __brimp("title", this); }
    get cookie() { return __brimp("cookie", this); }
    set cookie(value) { __brimp("setCookie", this, value); }
    get documentElement() { return __brimp("documentElement", this); }
    get head() { return __brimp("head", this); }
    get body() { return __brimp("body", this); }
    get implementation() { return __domImplementation; }
    createElement(name) { return __brimp("createElement", this, name); }
    createTextNode(text) { return __brimp("createTextNode", this, text); }
    createEvent(interfaceName) {
        switch (String(interfaceName).toLowerCase()) {
            case "event":
            case "events":
            case "htmlevents":
            case "svgevents": return new Event("");
            case "customevent": return new CustomEvent("");
            default: throw new DOMException("The event interface is not supported", "NotSupportedError");
        }
    }
    getElementById(id) { return __brimp("getElementById", this, id); }
    getElementsByTagName(name) {
        return new HTMLCollection(() => __brimp("getElementsByTagName", this, name));
    }
    getElementsByClassName(names) {
        return new HTMLCollection(() => __brimp("getElementsByClassName", this, names));
    }
    querySelector(selector) { return __brimp("querySelector", this, selector); }
    querySelectorAll(selector) { return new NodeList(__brimp("querySelectorAll", this, selector)); }
}

const __classLists = new WeakMap();

function __domTokenListTokens(element) {
    const value = element.getAttribute("class") || "";
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
    constructor(element) {
        this.__element = element;
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
    get length() { return __domTokenListTokens(this.__element).length; }
    get value() { return this.__element.getAttribute("class") || ""; }
    set value(value) { this.__element.setAttribute("class", String(value)); }
    item(index) { return __domTokenListTokens(this.__element)[Number(index)] ?? null; }
    contains(token) { return __domTokenListTokens(this.__element).includes(String(token)); }
    add(...tokens) {
        tokens = tokens.map(__validateDomToken);
        const values = __domTokenListTokens(this.__element);
        for (const token of tokens) if (!values.includes(token)) values.push(token);
        this.__element.setAttribute("class", values.join(" "));
    }
    remove(...tokens) {
        tokens = tokens.map(__validateDomToken);
        const remove = new Set(tokens);
        const values = __domTokenListTokens(this.__element);
        if (this.__element.getAttribute("class") !== null) {
            this.__element.setAttribute("class", values.filter(token => !remove.has(token)).join(" "));
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
        const values = __domTokenListTokens(this.__element);
        const index = values.indexOf(token);
        if (index === -1) return false;
        values[index] = newToken;
        this.__element.setAttribute("class", [...new Set(values)].join(" "));
        return true;
    }
    supports() { throw new TypeError("classList has no supported tokens"); }
    entries() { return __domTokenListTokens(this.__element).entries(); }
    keys() { return __domTokenListTokens(this.__element).keys(); }
    values() { return __domTokenListTokens(this.__element).values(); }
    forEach(callback, thisArg = undefined) {
        __domTokenListTokens(this.__element).forEach((value, index) => callback.call(thisArg, value, index, this));
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
        if (typeof items === "number") super(items);
        else super(...items);
    }
    item(index) { return this[Number(index)] ?? null; }
    get [Symbol.toStringTag]() { return "NodeList"; }
}

class Element extends Node {
    get tagName() { return __brimp("tagName", this); }
    get id() { return __brimp("getAttributeOrEmpty", this, "id"); }
    set id(value) { __brimp("setAttribute", this, "id", value); }
    get className() { return __brimp("getAttributeOrEmpty", this, "class"); }
    set className(value) { __brimp("setAttribute", this, "class", value); }
    get classList() {
        let list = __classLists.get(this);
        if (!list) { list = new DOMTokenList(this); __classLists.set(this, list); }
        return list;
    }
    set classList(value) {}
    get innerHTML() { return __brimp("innerHTML", this); }
    set innerHTML(value) { __brimp("setInnerHTML", this, value); }
    get style() { return __brimp("style", this); }
    get clientWidth() { return __brimp("clientWidth", this); }
    get clientHeight() { return __brimp("clientHeight", this); }
    get offsetWidth() { return __brimp("offsetWidth", this); }
    get offsetHeight() { return __brimp("offsetHeight", this); }
    getBoundingClientRect() {
        const rect = __brimp("boundingRect", this);
        return new DOMRect(rect[0], rect[1], rect[2], rect[3]);
    }
    getAttribute(name) { return __brimp("getAttribute", this, name); }
    setAttribute(name, value) { __brimp("setAttribute", this, name, value); }
    removeAttribute(name) { __brimp("removeAttribute", this, name); }
    hasAttribute(name) { return this.getAttribute(name) !== null; }
    getElementsByTagName(name) {
        return new HTMLCollection(() => __brimp("getElementsByTagName", this, name));
    }
    getElementsByClassName(names) {
        return new HTMLCollection(() => __brimp("getElementsByClassName", this, names));
    }
    querySelector(selector) { return __brimp("querySelector", this, selector); }
    querySelectorAll(selector) { return new NodeList(__brimp("querySelectorAll", this, selector)); }
    matches(selector) { return __brimp("matches", this, selector); }
    closest(selector) {
        let element = this;
        while (element) {
            if (element.matches(selector)) return element;
            element = element.parentElement;
        }
        return null;
    }
    click() { this.dispatchEvent(new Event("click", { bubbles: true, cancelable: true })); }
}

class HTMLElement extends Element {}
class HTMLAnchorElement extends HTMLElement {
    get href() { return __brimp("elementUrl", this, "href"); }
    set href(value) { this.setAttribute("href", value); }
    get origin() { return __brimp("elementUrl", this, "origin"); }
}
class HTMLBaseElement extends HTMLElement {
    get href() { return __brimp("elementUrl", this, "href"); }
    set href(value) { this.setAttribute("href", value); }
}
class Text extends Node {}
class Window extends EventTarget {}
Object.defineProperty(Window, Symbol.hasInstance, {
    value(object) { return object === globalThis; },
});
Object.defineProperties(Window.prototype, {
    innerWidth: { get() { return __brimp("innerWidth", this); } },
    innerHeight: { get() { return __brimp("innerHeight", this); } },
    devicePixelRatio: { get() { return __brimp("devicePixelRatio", this); } },
});

class Location {
    get href() { return __brimp("location", this, "href"); }
    get protocol() { return __brimp("location", this, "protocol"); }
    get host() { return __brimp("location", this, "host"); }
    get hostname() { return __brimp("location", this, "hostname"); }
    get port() { return __brimp("location", this, "port"); }
    get pathname() { return __brimp("location", this, "pathname"); }
    get search() { return __brimp("location", this, "search"); }
    get hash() { return __brimp("location", this, "hash"); }
    get origin() { return __brimp("location", this, "origin"); }
    toString() { return this.href; }
}

class Navigator {
    get userAgent() { return "Brimp/0.1"; }
    get platform() { return "MacIntel"; }
    get language() { return "en-US"; }
    get languages() { return ["en-US", "en"]; }
}

function __urlRecord(input, base = undefined) {
    try {
        return JSON.parse(__brimp("urlParse", window, String(input), base === undefined ? "" : String(base)));
    } catch (_) {
        throw new TypeError("Invalid URL");
    }
}

class URLSearchParams {
    constructor(init = "", owner = null) {
        this.__owner = owner;
        if (typeof init === "string") {
            this.__pairs = JSON.parse(__brimp("urlSearchParamsParse", window, init));
        } else if (init != null && typeof init[Symbol.iterator] === "function") {
            this.__pairs = Array.from(init, pair => {
                if (pair == null || typeof pair[Symbol.iterator] !== "function") throw new TypeError("each query pair must be iterable");
                const values = Array.from(pair);
                if (values.length !== 2) throw new TypeError("each query pair must have two items");
                return [String(values[0]), String(values[1])];
            });
        } else if (init != null) {
            this.__pairs = Object.keys(init).map(name => [name, String(init[name])]);
        } else {
            this.__pairs = [];
        }
    }
    get size() { return this.__pairs.length; }
    append(name, value) { this.__pairs.push([String(name), String(value)]); this.__changed(); }
    delete(name, value = undefined) {
        name = String(name);
        this.__pairs = this.__pairs.filter(pair => pair[0] !== name || (value !== undefined && pair[1] !== String(value)));
        this.__changed();
    }
    get(name) { name = String(name); const pair = this.__pairs.find(pair => pair[0] === name); return pair ? pair[1] : null; }
    getAll(name) { name = String(name); return this.__pairs.filter(pair => pair[0] === name).map(pair => pair[1]); }
    has(name, value = undefined) {
        name = String(name);
        return this.__pairs.some(pair => pair[0] === name && (value === undefined || pair[1] === String(value)));
    }
    set(name, value) {
        name = String(name); value = String(value);
        const index = this.__pairs.findIndex(pair => pair[0] === name);
        if (index === -1) this.__pairs.push([name, value]);
        else {
            this.__pairs[index][1] = value;
            this.__pairs = this.__pairs.filter((pair, item) => pair[0] !== name || item === index);
        }
        this.__changed();
    }
    sort() {
        this.__pairs = this.__pairs.map((pair, index) => [pair, index])
            .sort((a, b) => a[0][0] < b[0][0] ? -1 : a[0][0] > b[0][0] ? 1 : a[1] - b[1])
            .map(item => item[0]);
        this.__changed();
    }
    entries() { return this.__pairs.map(pair => pair.slice())[Symbol.iterator](); }
    keys() { return this.__pairs.map(pair => pair[0])[Symbol.iterator](); }
    values() { return this.__pairs.map(pair => pair[1])[Symbol.iterator](); }
    forEach(callback, thisArg = undefined) {
        for (const [name, value] of this.__pairs) callback.call(thisArg, value, name, this);
    }
    toString() { return __brimp("urlSearchParamsSerialize", window, JSON.stringify(this.__pairs)); }
    [Symbol.iterator]() { return this.entries(); }
    __changed() { if (this.__owner !== null) this.__owner.search = this.toString(); }
}

class URL {
    constructor(input, base = undefined) {
        this.__href = __urlRecord(input, base).href;
        this.__searchParams = new URLSearchParams(this.search, this);
    }
    static canParse(input, base = undefined) { try { __urlRecord(input, base); return true; } catch (_) { return false; } }
    static parse(input, base = undefined) { try { return new URL(input, base); } catch (_) { return null; } }
    get href() { return this.__href; }
    set href(value) { this.__set("href", value); }
    get origin() { return __urlRecord(this.__href).origin; }
    get protocol() { return __urlRecord(this.__href).protocol; }
    set protocol(value) { this.__set("protocol", value); }
    get username() { return __urlRecord(this.__href).username; }
    set username(value) { this.__set("username", value); }
    get password() { return __urlRecord(this.__href).password; }
    set password(value) { this.__set("password", value); }
    get host() { return __urlRecord(this.__href).host; }
    set host(value) { this.__set("host", value); }
    get hostname() { return __urlRecord(this.__href).hostname; }
    set hostname(value) { this.__set("hostname", value); }
    get port() { return __urlRecord(this.__href).port; }
    set port(value) { this.__set("port", value); }
    get pathname() { return __urlRecord(this.__href).pathname; }
    set pathname(value) { this.__set("pathname", value); }
    get search() { return __urlRecord(this.__href).search; }
    set search(value) { this.__set("search", value); }
    get searchParams() { return this.__searchParams; }
    get hash() { return __urlRecord(this.__href).hash; }
    set hash(value) { this.__set("hash", value); }
    toString() { return this.href; }
    toJSON() { return this.href; }
    __set(component, value) {
        this.__href = __brimp("urlSet", window, this.__href, component, String(value));
        if (this.__searchParams) {
            this.__searchParams.__pairs = JSON.parse(__brimp("urlSearchParamsParse", window, this.search));
        }
    }
}
globalThis.URL = URL;
globalThis.URLSearchParams = URLSearchParams;

class Headers {
    constructor(init = undefined) {
        this.__values = new Map();
        if (init instanceof Headers) {
            for (const [name, value] of init) this.append(name, value);
        } else if (Array.isArray(init)) {
            for (const pair of init) this.append(pair[0], pair[1]);
        } else if (init != null) {
            for (const name of Object.keys(init)) this.append(name, init[name]);
        }
    }
    append(name, value) {
        name = String(name).toLowerCase(); value = String(value);
        const old = this.__values.get(name);
        this.__values.set(name, old === undefined ? value : `${old}, ${value}`);
    }
    set(name, value) { this.__values.set(String(name).toLowerCase(), String(value)); }
    get(name) { return this.__values.get(String(name).toLowerCase()) ?? null; }
    has(name) { return this.__values.has(String(name).toLowerCase()); }
    delete(name) { this.__values.delete(String(name).toLowerCase()); }
    entries() { return this.__values.entries(); }
    keys() { return this.__values.keys(); }
    values() { return this.__values.values(); }
    forEach(callback, thisArg = undefined) {
        for (const [name, value] of this.__values) callback.call(thisArg, value, name, this);
    }
    [Symbol.iterator]() { return this.entries(); }
}

class Response {
    constructor(body = "", init = {}) {
        this.__body = String(body);
        this.status = Number(init.status ?? 200);
        this.statusText = String(init.statusText ?? "");
        this.headers = new Headers(init.headers);
        this.url = String(init.url ?? "");
        this.redirected = Boolean(init.redirected);
        this.type = "basic";
        this.bodyUsed = false;
    }
    get ok() { return this.status >= 200 && this.status <= 299; }
    text() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve(this.__body);
    }
    json() { return this.text().then(JSON.parse); }
}

globalThis.fetch = (input, init = {}) => {
    const url = String(input);
    const method = String(init.method ?? "GET").toUpperCase();
    const headers = new Headers(init.headers);
    const body = init.body == null ? null : String(init.body);
    return __brimp("fetch", window, url, method, JSON.stringify([...headers]), body)
        .then(serialized => {
            const payload = JSON.parse(serialized);
            return new Response(payload.body, payload);
        }, reason => { throw new TypeError(String(reason)); });
};

class DOMRect {
    constructor(x = 0, y = 0, width = 0, height = 0) {
        this.x = x; this.y = y; this.width = width; this.height = height;
        this.top = y; this.right = x + width; this.bottom = y + height; this.left = x;
    }
}

class CSSStyleDeclaration {
    getPropertyValue(name) { return __brimp("styleGetProperty", this, name); }
    setProperty(name, value) { __brimp("styleSetProperty", this, name, value); }
    removeProperty(name) { return __brimp("styleRemoveProperty", this, name); }
}

globalThis.DOMImplementation = DOMImplementation;
globalThis.DOMException = DOMException;
globalThis.Event = Event;
globalThis.CustomEvent = CustomEvent;
globalThis.AbortSignal = AbortSignal;
globalThis.AbortController = AbortController;
globalThis.DOMTokenList = DOMTokenList;
globalThis.HTMLCollection = HTMLCollection;
globalThis.NodeList = NodeList;
globalThis.HTMLAnchorElement = HTMLAnchorElement;
globalThis.HTMLBaseElement = HTMLBaseElement;

for (const [property, cssName] of [
    ["width", "width"],
    ["height", "height"],
    ["padding", "padding"],
    ["background", "background"],
    ["backgroundColor", "background-color"],
]) {
    Object.defineProperty(CSSStyleDeclaration.prototype, property, {
        configurable: true,
        enumerable: true,
        get() { return this.getPropertyValue(cssName); },
        set(value) { this.setProperty(cssName, value); },
    });
}

globalThis.window = globalThis;
globalThis.self = globalThis;
globalThis.addEventListener = EventTarget.prototype.addEventListener;
globalThis.removeEventListener = EventTarget.prototype.removeEventListener;
globalThis.dispatchEvent = EventTarget.prototype.dispatchEvent;
Object.defineProperties(globalThis, {
    innerWidth: { get: Object.getOwnPropertyDescriptor(Window.prototype, "innerWidth").get },
    innerHeight: { get: Object.getOwnPropertyDescriptor(Window.prototype, "innerHeight").get },
    devicePixelRatio: { get: Object.getOwnPropertyDescriptor(Window.prototype, "devicePixelRatio").get },
});
window.fetch = globalThis.fetch;
globalThis.location = Object.create(Location.prototype);
globalThis.navigator = Object.create(Navigator.prototype);
window.location = globalThis.location;
window.navigator = globalThis.navigator;
globalThis.getComputedStyle = element => __brimp("getComputedStyle", element);
globalThis.setTimeout = (callback, delay = 0) => __brimp("setTimeout", window, callback, delay);
globalThis.clearTimeout = id => __brimp("clearTimeout", window, id);
globalThis.queueMicrotask = callback => __brimp("queueMicrotask", window, callback);
window.setTimeout = globalThis.setTimeout;
window.clearTimeout = globalThis.clearTimeout;
window.queueMicrotask = globalThis.queueMicrotask;
"#;

pub struct TimerQueue {
    next_id: u32,
    timers: Vec<Timer>,
    microtasks: VecDeque<ProtectedJsObject>,
}

pub struct FetchQueue {
    next_id: u64,
    pending: VecDeque<PendingFetch>,
    settlements: HashMap<u64, PromiseSettlement>,
}

pub struct PendingFetch {
    pub id: u64,
    pub url: String,
    pub method: String,
    pub headers_json: String,
    pub body: Option<String>,
}

impl Default for FetchQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            pending: VecDeque::new(),
            settlements: HashMap::new(),
        }
    }
}

impl FetchQueue {
    fn push(&mut self, request: PendingFetch, settlement: PromiseSettlement) {
        self.settlements.insert(request.id, settlement);
        self.pending.push_back(request);
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    pub fn take_pending(&mut self) -> Vec<PendingFetch> {
        self.pending.drain(..).collect()
    }

    pub fn take_settlement(&mut self, id: u64) -> Option<PromiseSettlement> {
        self.settlements.remove(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.settlements.is_empty()
    }
}

struct Timer {
    id: u32,
    deadline: Instant,
    callback: ProtectedJsObject,
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            timers: Vec::new(),
            microtasks: VecDeque::new(),
        }
    }
}

impl TimerQueue {
    fn schedule(&mut self, delay_ms: f64, callback: ProtectedJsObject) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let delay_ms = if delay_ms.is_finite() {
            delay_ms.max(0.0)
        } else {
            0.0
        };
        self.timers.push(Timer {
            id,
            deadline: Instant::now() + Duration::from_secs_f64(delay_ms / 1000.0),
            callback,
        });
        id
    }

    fn clear(&mut self, id: u32) {
        self.timers.retain(|timer| timer.id != id);
    }

    pub fn pop_due(&mut self) -> Option<ProtectedJsObject> {
        let now = Instant::now();
        let index = self
            .timers
            .iter()
            .enumerate()
            .filter(|(_, timer)| timer.deadline <= now)
            .min_by_key(|(_, timer)| timer.deadline)
            .map(|(index, _)| index)?;
        Some(self.timers.remove(index).callback)
    }

    fn queue_microtask(&mut self, callback: ProtectedJsObject) {
        self.microtasks.push_back(callback);
    }

    pub fn pop_microtask(&mut self) -> Option<ProtectedJsObject> {
        self.microtasks.pop_front()
    }
}

pub struct BindingRuntime {
    state: Rc<BindingState>,
}

#[derive(Default)]
pub struct BrowsingContext {
    url: Mutex<Option<String>>,
    cookies: Mutex<cookie_store::CookieStore>,
    request_headers: Mutex<Vec<(http::HeaderName, http::HeaderValue)>>,
}

impl BrowsingContext {
    pub fn set_request_identity(&self, user_agent: &str, locale: &str) -> Result<(), String> {
        let headers = vec![
            (
                http::header::USER_AGENT,
                http::HeaderValue::from_str(user_agent).map_err(|error| error.to_string())?,
            ),
            (
                http::header::ACCEPT_LANGUAGE,
                http::HeaderValue::from_str(locale).map_err(|error| error.to_string())?,
            ),
        ];
        *self
            .request_headers
            .lock()
            .expect("request identity lock poisoned") = headers;
        Ok(())
    }

    pub fn apply_request_identity(&self, headers: &mut network::HeaderList) {
        for (name, value) in self
            .request_headers
            .lock()
            .expect("request identity lock poisoned")
            .iter()
        {
            if !headers.contains_key(name) {
                headers.insert(name.clone(), value.clone());
            }
        }
    }

    pub fn set_url(&self, url: impl Into<String>) {
        *self.url.lock().expect("browsing URL lock poisoned") = Some(url.into());
    }

    fn current_url(&self) -> Option<String> {
        self.url.lock().expect("browsing URL lock poisoned").clone()
    }

    pub fn store_response_cookie(&self, url: &str, header: &str) {
        let (Ok(url), Ok(cookie)) = (
            url::Url::parse(url),
            cookie_store::RawCookie::parse(header.to_owned()),
        ) else {
            return;
        };
        self.cookies
            .lock()
            .expect("cookie store lock poisoned")
            .store_response_cookies(std::iter::once(cookie), &url);
    }

    pub fn cookie_header(&self, url: &str) -> Option<String> {
        let url = url::Url::parse(url).ok()?;
        let value = self
            .cookies
            .lock()
            .expect("cookie store lock poisoned")
            .get_request_values(&url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        (!value.is_empty()).then_some(value)
    }

    pub fn cookies_for_url(&self, url: &str) -> Vec<(String, String)> {
        let Ok(url) = url::Url::parse(url) else {
            return Vec::new();
        };
        self.cookies
            .lock()
            .expect("cookie store lock poisoned")
            .get_request_values(&url)
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect()
    }

    fn document_cookies(&self) -> String {
        let raw_url = self.url.lock().expect("browsing URL lock poisoned");
        let Some(url) = raw_url.as_deref().and_then(|url| url::Url::parse(url).ok()) else {
            return String::new();
        };
        self.cookies
            .lock()
            .expect("cookie store lock poisoned")
            .matches(&url)
            .into_iter()
            .filter(|cookie| cookie.http_only() != Some(true))
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn set_document_cookie(&self, header: &str) {
        let url = self.url.lock().expect("browsing URL lock poisoned").clone();
        if let Some(url) = url {
            self.store_response_cookie(&url, header);
        }
    }
}

struct BindingState {
    document: Rc<RefCell<BrowserDocument>>,
    wrappers: WrapperCache,
    style_wrappers: WrapperCache,
    computed_style_wrappers: WrapperCache,
    prototypes: RefCell<Option<Prototypes>>,
    timers: Rc<RefCell<TimerQueue>>,
    browsing_context: Arc<BrowsingContext>,
    fetches: Rc<RefCell<FetchQueue>>,
}

struct Prototypes {
    node: ProtectedJsObject,
    document: ProtectedJsObject,
    html_element: ProtectedJsObject,
    html_anchor_element: ProtectedJsObject,
    html_base_element: ProtectedJsObject,
    text: ProtectedJsObject,
    css_style: ProtectedJsObject,
}

impl BindingRuntime {
    pub fn install(
        runtime: &JsRuntime,
        document: Rc<RefCell<BrowserDocument>>,
        timers: Rc<RefCell<TimerQueue>>,
        browsing_context: Arc<BrowsingContext>,
        fetches: Rc<RefCell<FetchQueue>>,
    ) -> Result<Self, JsException> {
        let state = Rc::new(BindingState {
            document,
            wrappers: WrapperCache::default(),
            style_wrappers: WrapperCache::default(),
            computed_style_wrappers: WrapperCache::default(),
            prototypes: RefCell::new(None),
            timers,
            browsing_context,
            fetches,
        });
        let callback_state = Rc::clone(&state);
        runtime.set_global_function("__brimp", move |call| dispatch(&callback_state, &call))?;
        runtime.eval(CLASS_DEFINITIONS)?;

        *state.prototypes.borrow_mut() = Some(Prototypes {
            node: runtime.eval("Node.prototype")?.to_object()?,
            document: runtime.eval("Document.prototype")?.to_object()?,
            html_element: runtime.eval("HTMLElement.prototype")?.to_object()?,
            html_anchor_element: runtime.eval("HTMLAnchorElement.prototype")?.to_object()?,
            html_base_element: runtime.eval("HTMLBaseElement.prototype")?.to_object()?,
            text: runtime.eval("Text.prototype")?.to_object()?,
            css_style: runtime.eval("CSSStyleDeclaration.prototype")?.to_object()?,
        });
        let bindings = Self { state };
        bindings.reset_document(runtime)?;
        Ok(bindings)
    }

    pub fn reset_document(&self, runtime: &JsRuntime) -> Result<(), JsException> {
        self.state.wrappers.clear();
        self.state.style_wrappers.clear();
        self.state.computed_style_wrappers.clear();
        let document_id = self.state.document.borrow().root().id;
        let prototype = self
            .state
            .prototypes
            .borrow()
            .as_ref()
            .expect("bindings are initialized")
            .document
            .identity();
        let document =
            self.state
                .wrappers
                .wrap_with_runtime_prototype(runtime, document_id, prototype);
        runtime.set_global_object("document", &document)?;
        runtime.eval("window.document = document")?;
        Ok(())
    }

    pub fn wrapper_cache(&self) -> &WrapperCache {
        &self.state.wrappers
    }
}

fn dispatch(state: &BindingState, call: &NativeCall<'_>) -> Result<NativeValue, NativeError> {
    let operation = required_string(call, 0, "operation")?;
    match operation.as_str() {
        "setTimeout" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing timer callback"))?
                .to_function()?;
            let delay = call
                .argument(3)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0);
            let id = state.timers.borrow_mut().schedule(delay, callback);
            Ok(NativeValue::Number(f64::from(id)))
        }
        "clearTimeout" => {
            let id = call
                .argument(2)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0) as u32;
            state.timers.borrow_mut().clear(id);
            Ok(NativeValue::Undefined)
        }
        "queueMicrotask" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing microtask callback"))?
                .to_function()?;
            state.timers.borrow_mut().queue_microtask(callback);
            Ok(NativeValue::Undefined)
        }
        "location" => {
            let property = required_string(call, 2, "location property")?;
            let raw_url = state
                .browsing_context
                .url
                .lock()
                .expect("browsing URL lock poisoned");
            let Some(raw_url) = raw_url.as_deref() else {
                return Ok(NativeValue::String(String::new()));
            };
            let url = url::Url::parse(raw_url).map_err(err)?;
            let value = match property.as_str() {
                "href" => url.as_str().to_string(),
                "protocol" => format!("{}:", url.scheme()),
                "host" => match (url.host_str(), url.port()) {
                    (Some(host), Some(port)) => format!("{host}:{port}"),
                    (Some(host), None) => host.to_string(),
                    (None, _) => String::new(),
                },
                "hostname" => url.host_str().unwrap_or_default().to_string(),
                "port" => url.port().map(|port| port.to_string()).unwrap_or_default(),
                "pathname" => url.path().to_string(),
                "search" => url
                    .query()
                    .map(|query| format!("?{query}"))
                    .unwrap_or_default(),
                "hash" => url
                    .fragment()
                    .map(|hash| format!("#{hash}"))
                    .unwrap_or_default(),
                "origin" => url.origin().ascii_serialization(),
                _ => return Err(NativeError::new("unknown Location property")),
            };
            Ok(NativeValue::String(value))
        }
        "urlParse" => {
            let input = required_string(call, 2, "URL input")?;
            let base = required_string(call, 3, "URL base")?;
            let base = (!base.is_empty())
                .then(|| url::Url::parse(&base).map_err(err))
                .transpose()?;
            let parsed = url::Url::options()
                .base_url(base.as_ref())
                .parse(&input)
                .map_err(err)?;
            Ok(NativeValue::String(url_record_json(&parsed)?))
        }
        "urlSet" => {
            let href = required_string(call, 2, "URL href")?;
            let component = required_string(call, 3, "URL component")?;
            let value = required_string(call, 4, "URL component value")?;
            Ok(NativeValue::String(set_url_component(
                &href, &component, &value,
            )?))
        }
        "urlSearchParamsParse" => {
            let input = required_string(call, 2, "query")?;
            let pairs = url::form_urlencoded::parse(input.trim_start_matches('?').as_bytes())
                .into_owned()
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&pairs).map_err(err)?,
            ))
        }
        "urlSearchParamsSerialize" => {
            let input = required_string(call, 2, "query pairs")?;
            let pairs: Vec<(String, String)> = serde_json::from_str(&input).map_err(err)?;
            let output = url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish();
            Ok(NativeValue::String(output))
        }
        "fetch" => {
            let url = required_string(call, 2, "fetch URL")?;
            let method = required_string(call, 3, "fetch method")?;
            let headers_json = required_string(call, 4, "fetch headers")?;
            let body = call
                .argument(5)
                .filter(|value| !value.is_null_or_undefined())
                .map(|value| value.to_string())
                .transpose()?;
            let (promise, settlement) = call.make_deferred_promise()?.into_parts();
            let mut fetches = state.fetches.borrow_mut();
            let id = fetches.next_id();
            fetches.push(
                PendingFetch {
                    id,
                    url,
                    method,
                    headers_json,
                    body,
                },
                settlement,
            );
            Ok(NativeValue::ProtectedObject(promise))
        }
        "innerWidth" | "innerHeight" | "devicePixelRatio" => {
            let metrics = state.document.borrow().viewport_metrics();
            let index = match operation.as_str() {
                "innerWidth" => 0,
                "innerHeight" => 1,
                _ => 2,
            };
            Ok(NativeValue::Number(metrics[index]))
        }
        "documentElement" => {
            let id = state
                .document
                .borrow()
                .blitz()
                .try_root_element()
                .map(|node| node.id);
            optional_node(state, call, id)
        }
        "title" => {
            required_document_target(state, call)?;
            let document = state.document.borrow();
            let title = document
                .query_selector("title")
                .map_err(err)?
                .and_then(|id| document.node(id))
                .map(|node| node.text_content())
                .unwrap_or_default();
            Ok(NativeValue::String(title))
        }
        "cookie" => {
            required_document_target(state, call)?;
            Ok(NativeValue::String(
                state.browsing_context.document_cookies(),
            ))
        }
        "setCookie" => {
            required_document_target(state, call)?;
            let cookie = required_string(call, 2, "cookie")?;
            state.browsing_context.set_document_cookie(&cookie);
            Ok(NativeValue::Undefined)
        }
        "head" => {
            let id = state
                .document
                .borrow()
                .query_selector("head")
                .map_err(err)?;
            optional_node(state, call, id)
        }
        "body" => {
            let id = state
                .document
                .borrow()
                .query_selector("body")
                .map_err(err)?;
            optional_node(state, call, id)
        }
        "createElement" => {
            required_document_target(state, call)?;
            let tag = required_string(call, 2, "tag name")?.to_ascii_lowercase();
            if tag.is_empty() {
                return Err(NativeError::new("tag name cannot be empty"));
            }
            let name = QualName::new(None, ns!(html), LocalName::from(tag));
            let id = state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .create_element(name, vec![]);
            node_value(state, call, id)
        }
        "createTextNode" => {
            required_document_target(state, call)?;
            let text = required_string(call, 2, "text")?;
            let id = state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .create_text_node(&text);
            node_value(state, call, id)
        }
        "getElementById" => {
            required_document_target(state, call)?;
            let id = required_string(call, 2, "id")?;
            let node = state.document.borrow().get_element_by_id(&id);
            optional_node(state, call, node)
        }
        "getElementsByTagName" => {
            let root_id = required_parent_node_target(state, call)?;
            let name = required_string(call, 2, "name")?.to_ascii_lowercase();
            let document = state.document.borrow();
            let nodes = descendant_ids(&document, root_id)?
                .into_iter()
                .filter(|id| {
                    document.node(*id).is_some_and(|node| match &node.data {
                        NodeData::Element(element) => {
                            name == "*" || element.name.local.as_ref() == name
                        }
                        _ => false,
                    })
                })
                .collect::<Vec<_>>();
            drop(document);
            node_array(state, call, &nodes)
        }
        "getElementsByClassName" => {
            let root_id = required_parent_node_target(state, call)?;
            let names = required_string(call, 2, "class names")?
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let document = state.document.borrow();
            let nodes = if names.is_empty() {
                Vec::new()
            } else {
                descendant_ids(&document, root_id)?
                    .into_iter()
                    .filter(|id| {
                        document
                            .node(*id)
                            .and_then(|node| node.element_data())
                            .and_then(|element| element.attr(LocalName::from("class")))
                            .is_some_and(|classes| {
                                let classes =
                                    classes.split_ascii_whitespace().collect::<HashSet<_>>();
                                names.iter().all(|name| classes.contains(name.as_str()))
                            })
                    })
                    .collect()
            };
            drop(document);
            node_array(state, call, &nodes)
        }
        "querySelector" => {
            let root_id = required_parent_node_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            let document = state.document.borrow();
            let descendants = descendant_ids(&document, root_id)?
                .into_iter()
                .collect::<HashSet<_>>();
            let node = document
                .query_selector_all(&selector)
                .map_err(err)?
                .into_iter()
                .find(|id| descendants.contains(id));
            drop(document);
            optional_node(state, call, node)
        }
        "querySelectorAll" => {
            let root_id = required_parent_node_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            let document = state.document.borrow();
            let descendants = descendant_ids(&document, root_id)?
                .into_iter()
                .collect::<HashSet<_>>();
            let nodes = document
                .query_selector_all(&selector)
                .map_err(err)?
                .into_iter()
                .filter(|id| descendants.contains(id))
                .collect::<Vec<_>>();
            drop(document);
            node_array(state, call, &nodes)
        }
        "matches" => {
            let id = required_element_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            Ok(NativeValue::Boolean(
                state
                    .document
                    .borrow()
                    .query_selector_all(&selector)
                    .map_err(err)?
                    .contains(&id),
            ))
        }
        "nodeType" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let node_type = match node.data {
                NodeData::Element(_) | NodeData::AnonymousBlock(_) => 1.0,
                NodeData::Text(_) => 3.0,
                NodeData::Comment => 8.0,
                NodeData::Document => 9.0,
            };
            Ok(NativeValue::Number(node_type))
        }
        "nodeName" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let name = match &node.data {
                NodeData::Element(element) | NodeData::AnonymousBlock(element) => {
                    element.name.local.to_string().to_ascii_uppercase()
                }
                NodeData::Text(_) => "#text".to_owned(),
                NodeData::Comment => "#comment".to_owned(),
                NodeData::Document => "#document".to_owned(),
            };
            Ok(NativeValue::String(name))
        }
        "parentNode" => {
            let id = required_node_target(state, call)?;
            let parent = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .parent;
            optional_node(state, call, parent)
        }
        "firstChild" => {
            let id = required_node_target(state, call)?;
            let child = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .first()
                .copied();
            optional_node(state, call, child)
        }
        "lastChild" => {
            let id = required_node_target(state, call)?;
            let child = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .last()
                .copied();
            optional_node(state, call, child)
        }
        "childNodes" => {
            let id = required_node_target(state, call)?;
            let children = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .clone();
            node_array(state, call, &children)
        }
        "textContent" => {
            let id = required_node_target(state, call)?;
            let text = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .text_content();
            Ok(NativeValue::String(text))
        }
        "setTextContent" => {
            let id = required_node_target(state, call)?;
            let value = required_string(call, 2, "textContent")?;
            set_text_content(state, id, &value)?;
            Ok(NativeValue::Undefined)
        }
        "appendChild" => mutate_child(state, call, ChildMutation::Append),
        "removeChild" => mutate_child(state, call, ChildMutation::Remove),
        "insertBefore" => mutate_child(state, call, ChildMutation::InsertBefore),
        "tagName" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            Ok(NativeValue::String(
                element.name.local.to_string().to_ascii_uppercase(),
            ))
        }
        "getAttribute" | "getAttributeOrEmpty" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let document = state.document.borrow();
            let value = document
                .node(id)
                .and_then(|node| node.element_data())
                .and_then(|element| element.attr(LocalName::from(name)));
            match (operation.as_str(), value) {
                ("getAttributeOrEmpty", None) => Ok(NativeValue::String(String::new())),
                (_, None) => Ok(NativeValue::Null),
                (_, Some(value)) => Ok(NativeValue::String(value.to_owned())),
            }
        }
        "setAttribute" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let value = required_string(call, 3, "attribute value")?;
            let name = QualName::new(None, ns!(), LocalName::from(name));
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .set_attribute(id, name, &value);
            Ok(NativeValue::Undefined)
        }
        "removeAttribute" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let name = QualName::new(None, ns!(), LocalName::from(name));
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .clear_attribute(id, name);
            Ok(NativeValue::Undefined)
        }
        "elementUrl" => {
            let id = required_element_target(state, call)?;
            let property = required_string(call, 2, "URL property")?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let input = element
                .attr(LocalName::from("href"))
                .unwrap_or_default()
                .to_owned();
            let document_url = state
                .browsing_context
                .current_url()
                .and_then(|url| url::Url::parse(&url).ok());
            let base_url = if element.name.local.as_ref() == "base" {
                document_url.clone()
            } else {
                document
                    .query_selector("base[href]")
                    .ok()
                    .flatten()
                    .and_then(|base_id| {
                        document
                            .node(base_id)
                            .and_then(|node| node.element_data())
                            .and_then(|base| base.attr(LocalName::from("href")))
                    })
                    .and_then(|base| {
                        url::Url::options()
                            .base_url(document_url.as_ref())
                            .parse(base)
                            .ok()
                    })
                    .or(document_url)
            };
            let parsed = url::Url::options()
                .base_url(base_url.as_ref())
                .parse(&input)
                .map_err(err)?;
            let value = match property.as_str() {
                "href" => parsed.as_str().to_owned(),
                "origin" => parsed.origin().ascii_serialization(),
                _ => return Err(NativeError::new("unknown element URL property")),
            };
            Ok(NativeValue::String(value))
        }
        "innerHTML" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let mut html = String::new();
            for child in &node.children {
                document
                    .node(*child)
                    .ok_or_else(stale_wrapper)?
                    .write_outer_html(&mut html);
            }
            Ok(NativeValue::String(html))
        }
        "setInnerHTML" => {
            let id = required_element_target(state, call)?;
            let html = required_string(call, 2, "innerHTML")?;
            let removed = descendant_ids(&state.document.borrow(), id)?;
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .set_inner_html(id, &html);
            state.wrappers.remove_nodes(&removed);
            state.style_wrappers.remove_nodes(&removed);
            Ok(NativeValue::Undefined)
        }
        "style" => {
            let id = required_element_target(state, call)?;
            let prototype = prototypes(state).css_style.identity();
            let style = state
                .style_wrappers
                .wrap_with_prototype(call, id, prototype);
            Ok(NativeValue::Object(style))
        }
        "getComputedStyle" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let prototype = prototypes(state).css_style.identity();
            let style = state
                .computed_style_wrappers
                .wrap_with_prototype(call, id, prototype);
            Ok(NativeValue::Object(style))
        }
        "styleGetProperty" => {
            let name = required_string(call, 2, "property name")?;
            let object = required_object(call, 1, "style receiver")?;
            if let Some(id) = state.style_wrappers.node_id(object) {
                Ok(NativeValue::String(inline_style_property(state, id, &name)))
            } else if let Some(id) = state.computed_style_wrappers.node_id(object) {
                resolve_document(state);
                Ok(NativeValue::String(
                    state
                        .document
                        .borrow()
                        .computed_style_property(id, &name)
                        .unwrap_or_default(),
                ))
            } else {
                Err(NativeError::new("receiver is not a CSSStyleDeclaration"))
            }
        }
        "styleSetProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property name")?;
            let value = required_string(call, 3, "property value")?;
            state
                .document
                .borrow_mut()
                .set_style_property(id, &name, &value);
            Ok(NativeValue::Undefined)
        }
        "styleRemoveProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property name")?;
            let old = inline_style_property(state, id, &name);
            state.document.borrow_mut().remove_style_property(id, &name);
            Ok(NativeValue::String(old))
        }
        "clientWidth" | "clientHeight" | "offsetWidth" | "offsetHeight" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let document = state.document.borrow();
            let size = if operation.starts_with("client") {
                document.client_size(id)
            } else {
                document.offset_size(id)
            }
            .ok_or_else(stale_wrapper)?;
            let index = usize::from(operation.ends_with("Height"));
            Ok(NativeValue::Number(size[index]))
        }
        "boundingRect" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let rect = state
                .document
                .borrow()
                .bounding_rect(id)
                .ok_or_else(stale_wrapper)?;
            let values = rect.into_iter().map(NativeValue::Number).collect();
            Ok(NativeValue::ProtectedObject(call.make_value_array(values)?))
        }
        _ => Err(NativeError::new(format!(
            "unknown native DOM operation: {operation}"
        ))),
    }
}

fn prototypes(state: &BindingState) -> std::cell::Ref<'_, Prototypes> {
    std::cell::Ref::map(state.prototypes.borrow(), |prototypes| {
        prototypes.as_ref().expect("bindings are initialized")
    })
}

fn required_string(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<String, NativeError> {
    call.argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_string()
}

fn required_object(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<JsObjectIdentity, NativeError> {
    call.argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .as_object()?
        .ok_or_else(|| NativeError::new(format!("{label} must be an object")))
}

fn required_node_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, 1, "node receiver")?;
    state
        .wrappers
        .node_id(object)
        .ok_or_else(|| NativeError::new("receiver is not a native Node"))
}

fn required_document_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    let is_document = state
        .document
        .borrow()
        .node(id)
        .is_some_and(|node| matches!(node.data, NodeData::Document));
    if is_document {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not a Document"))
    }
}

fn required_parent_node_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    let is_parent_node = state.document.borrow().node(id).is_some_and(|node| {
        matches!(
            node.data,
            NodeData::Document | NodeData::Element(_) | NodeData::AnonymousBlock(_)
        )
    });
    if is_parent_node {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not a ParentNode"))
    }
}

fn required_element_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    if state
        .document
        .borrow()
        .node(id)
        .is_some_and(|node| node.is_element())
    {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not an Element"))
    }
}

fn required_style_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, 1, "style receiver")?;
    state
        .style_wrappers
        .node_id(object)
        .ok_or_else(|| NativeError::new("receiver is not a CSSStyleDeclaration"))
}

fn optional_node(
    state: &BindingState,
    call: &NativeCall<'_>,
    node: Option<NodeId>,
) -> Result<NativeValue, NativeError> {
    match node {
        Some(node) => node_value(state, call, node),
        None => Ok(NativeValue::Null),
    }
}

fn node_value(
    state: &BindingState,
    call: &NativeCall<'_>,
    node_id: NodeId,
) -> Result<NativeValue, NativeError> {
    let prototype = {
        let document = state.document.borrow();
        let node = document.node(node_id).ok_or_else(stale_wrapper)?;
        let prototypes = prototypes(state);
        match node.data {
            NodeData::Document => prototypes.document.identity(),
            NodeData::Element(ref element) | NodeData::AnonymousBlock(ref element) => {
                match element.name.local.as_ref() {
                    "a" => prototypes.html_anchor_element.identity(),
                    "base" => prototypes.html_base_element.identity(),
                    _ => prototypes.html_element.identity(),
                }
            }
            NodeData::Text(_) => prototypes.text.identity(),
            NodeData::Comment => prototypes.node.identity(),
        }
    };
    Ok(NativeValue::Object(
        state.wrappers.wrap_with_prototype(call, node_id, prototype),
    ))
}

fn node_array(
    state: &BindingState,
    call: &NativeCall<'_>,
    nodes: &[NodeId],
) -> Result<NativeValue, NativeError> {
    let wrappers = nodes
        .iter()
        .map(|node| match node_value(state, call, *node)? {
            NativeValue::Object(object) => Ok(object),
            _ => unreachable!(),
        })
        .collect::<Result<Vec<_>, NativeError>>()?;
    Ok(NativeValue::ProtectedObject(call.make_array(&wrappers)?))
}

enum ChildMutation {
    Append,
    Remove,
    InsertBefore,
}

fn mutate_child(
    state: &BindingState,
    call: &NativeCall<'_>,
    mutation: ChildMutation,
) -> Result<NativeValue, NativeError> {
    let parent = required_node_target(state, call)?;
    let child_object = required_object(call, 2, "child")?;
    let child = state
        .wrappers
        .node_id(child_object)
        .ok_or_else(|| NativeError::new("child is not a native Node"))?;
    ensure_can_parent(&state.document.borrow(), parent, child)?;

    match mutation {
        ChildMutation::Append => {
            let mut document = state.document.borrow_mut();
            let mut mutator = document.blitz_mut().mutate();
            if mutator.node_has_parent(child) {
                mutator.remove_node(child);
            }
            mutator.append_children(parent, &[child]);
        }
        ChildMutation::Remove => {
            let mut document = state.document.borrow_mut();
            let mut mutator = document.blitz_mut().mutate();
            if mutator.parent_id(child) != Some(parent) {
                return Err(NativeError::new("node is not a child of this parent"));
            }
            mutator.remove_node(child);
        }
        ChildMutation::InsertBefore => {
            let reference = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing reference node"))?;
            if reference.is_null_or_undefined() {
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                if mutator.node_has_parent(child) {
                    mutator.remove_node(child);
                }
                mutator.append_children(parent, &[child]);
            } else {
                let reference = reference
                    .as_object()?
                    .and_then(|object| state.wrappers.node_id(object))
                    .ok_or_else(|| NativeError::new("reference is not a native Node"))?;
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                if mutator.parent_id(reference) != Some(parent) {
                    return Err(NativeError::new(
                        "reference node is not a child of this parent",
                    ));
                }
                if reference != child {
                    if mutator.node_has_parent(child) {
                        mutator.remove_node(child);
                    }
                    mutator.insert_nodes_before(reference, &[child]);
                }
            }
        }
    }
    Ok(NativeValue::Object(child_object))
}

fn ensure_can_parent(
    document: &BrowserDocument,
    parent: NodeId,
    child: NodeId,
) -> Result<(), NativeError> {
    if parent == child {
        return Err(NativeError::new("a node cannot contain itself"));
    }
    let mut ancestor = document.node(parent).and_then(|node| node.parent);
    while let Some(id) = ancestor {
        if id == child {
            return Err(NativeError::new("operation would create a DOM cycle"));
        }
        ancestor = document.node(id).and_then(|node| node.parent);
    }
    Ok(())
}

fn set_text_content(state: &BindingState, node_id: NodeId, value: &str) -> Result<(), NativeError> {
    let is_text = state
        .document
        .borrow()
        .node(node_id)
        .ok_or_else(stale_wrapper)?
        .is_text_node();
    if is_text {
        state
            .document
            .borrow_mut()
            .blitz_mut()
            .mutate()
            .set_node_text(node_id, value);
        return Ok(());
    }

    let removed = descendant_ids(&state.document.borrow(), node_id)?;
    let mut document = state.document.borrow_mut();
    let mut mutator = document.blitz_mut().mutate();
    mutator.remove_and_drop_all_children(node_id);
    if !value.is_empty() {
        let text = mutator.create_text_node(value);
        mutator.append_children(node_id, &[text]);
    }
    drop(mutator);
    drop(document);
    state.wrappers.remove_nodes(&removed);
    state.style_wrappers.remove_nodes(&removed);
    Ok(())
}

fn descendant_ids(document: &BrowserDocument, node_id: NodeId) -> Result<Vec<NodeId>, NativeError> {
    fn collect(document: &BrowserDocument, node_id: NodeId, output: &mut Vec<NodeId>) {
        if let Some(node) = document.node(node_id) {
            for child in &node.children {
                output.push(*child);
                collect(document, *child, output);
            }
        }
    }
    if document.node(node_id).is_none() {
        return Err(stale_wrapper());
    }
    let mut output = Vec::new();
    collect(document, node_id, &mut output);
    Ok(output)
}

fn inline_style_property(state: &BindingState, node_id: NodeId, name: &str) -> String {
    state
        .document
        .borrow()
        .inline_style_css(node_id)
        .as_deref()
        .and_then(|style| {
            style.split(';').find_map(|declaration| {
                let (property, value) = declaration.split_once(':')?;
                property
                    .trim()
                    .eq_ignore_ascii_case(name)
                    .then(|| value.trim().to_owned())
            })
        })
        .unwrap_or_default()
}

fn resolve_document(state: &BindingState) {
    state.document.borrow_mut().resolve();
}

fn url_record_json(url: &url::Url) -> Result<String, NativeError> {
    let host = match (url.host_str(), url.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_owned(),
        (None, _) => String::new(),
    };
    serde_json::to_string(&serde_json::json!({
        "href": url.as_str(),
        "origin": url.origin().ascii_serialization(),
        "protocol": format!("{}:", url.scheme()),
        "username": url.username(),
        "password": url.password().unwrap_or_default(),
        "host": host,
        "hostname": url.host_str().unwrap_or_default(),
        "port": url.port().map(|port| port.to_string()).unwrap_or_default(),
        "pathname": url.path(),
        "search": url.query().map(|query| format!("?{query}")).unwrap_or_default(),
        "hash": url.fragment().map(|fragment| format!("#{fragment}")).unwrap_or_default(),
    }))
    .map_err(err)
}

fn set_url_component(href: &str, component: &str, value: &str) -> Result<String, NativeError> {
    if component == "href" {
        return url::Url::parse(value).map(|url| url.into()).map_err(err);
    }
    let mut url = url::Url::parse(href).map_err(err)?;
    match component {
        "protocol" => {
            let _ = url.set_scheme(value.trim_end_matches(':'));
        }
        "username" => {
            let _ = url.set_username(value);
        }
        "password" => {
            let _ = url.set_password(Some(value));
        }
        "host" => {
            if let Ok(host_url) = url::Url::parse(&format!("{}://{value}/", url.scheme())) {
                let _ = url.set_host(host_url.host_str());
                let _ = url.set_port(host_url.port());
            }
        }
        "hostname" => {
            let _ = url.set_host(Some(value));
        }
        "port" => {
            let port = if value.is_empty() {
                None
            } else if let Ok(port) = value.parse::<u16>() {
                Some(port)
            } else {
                return Ok(url.into());
            };
            let _ = url.set_port(port);
        }
        "pathname" => url.set_path(value),
        "search" => url.set_query((!value.is_empty()).then(|| value.trim_start_matches('?'))),
        "hash" => url.set_fragment((!value.is_empty()).then(|| value.trim_start_matches('#'))),
        _ => return Err(NativeError::new("unknown URL component")),
    }
    Ok(url.into())
}

fn stale_wrapper() -> NativeError {
    NativeError::new("native node no longer exists")
}

fn err(error: impl ToString) -> NativeError {
    NativeError::new(error)
}
