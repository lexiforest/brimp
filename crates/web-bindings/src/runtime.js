(() => {
"use strict";

const __callHost = globalThis.__brimp;
delete globalThis.__brimp;
const __runtimeFeatures = JSON.parse(__callHost("runtimeFeatures"));

const __initialGlobalNames = new Set(Object.getOwnPropertyNames(globalThis));
const __nativeFunctions = new WeakSet();
const __nativeSources = new WeakMap();
const __originalFunctionToString = Function.prototype.toString;
const __functionToString = {
    toString() {
        const exactSource = __nativeSources.get(this);
        if (exactSource !== undefined) return exactSource;
        if (__nativeFunctions.has(this)) {
            return `function ${this.name || ""}() { [native code] }`;
        }
        return __originalFunctionToString.call(this);
    },
}.toString;

function __markWebBuiltin(fn, source = undefined) {
    if (typeof fn !== "function") return fn;
    if (source === undefined) __nativeFunctions.add(fn);
    else __nativeSources.set(fn, source);
    return fn;
}

function __markWebBuiltinInterface(constructor) {
    if (typeof constructor !== "function") return;
    __markWebBuiltin(constructor);
    const prototype = constructor.prototype;
    if (prototype === null || typeof prototype !== "object") return;
    for (const key of Reflect.ownKeys(prototype)) {
        const descriptor = Object.getOwnPropertyDescriptor(prototype, key);
        if (descriptor === undefined) continue;
        if (typeof descriptor.value === "function") __markWebBuiltin(descriptor.value);
        if (typeof descriptor.get === "function") {
            __markWebBuiltin(descriptor.get, `function get ${String(key)}() { [native code] }`);
        }
        if (typeof descriptor.set === "function") {
            __markWebBuiltin(descriptor.set, `function set ${String(key)}() { [native code] }`);
        }
    }
}

Object.defineProperty(Function.prototype, "toString", {
    value: __functionToString,
    writable: true,
    enumerable: false,
    configurable: true,
});
__markWebBuiltin(__functionToString);

// Persona installation runs immediately after these bindings. Keep the marker
// non-enumerable during that hand-off; persona.js removes it before page code runs.
Object.defineProperty(globalThis, "__brimpMarkWebBuiltin", {
    value: __markWebBuiltin,
    writable: false,
    enumerable: false,
    configurable: true,
});

const __eventListeners = new WeakMap();
const __disconnectedNodeOrder = new WeakMap();
const __attributeMaps = new WeakMap();
const __inputValues = new WeakMap();
const __iframeBrowsingContexts = new WeakMap();
const __templateContents = new WeakMap();
let __activeElement = null;
let __nextDisconnectedNodeOrder = 1;

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

class UIEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        this.view = options.view === undefined ? null : options.view;
        this.detail = Number(options.detail ?? 0);
        this.sourceCapabilities = null;
        this.which = 0;
    }
    initUIEvent(type, bubbles = false, cancelable = false, view = null, detail = 0) {
        if (this.__dispatching) return;
        this.initEvent(type, bubbles, cancelable);
        this.view = view;
        this.detail = Number(detail);
    }
}

function __initializeModifiers(event, options) {
    event.ctrlKey = Boolean(options.ctrlKey);
    event.shiftKey = Boolean(options.shiftKey);
    event.altKey = Boolean(options.altKey);
    event.metaKey = Boolean(options.metaKey);
    event.__modifierAltGraph = Boolean(options.modifierAltGraph);
    event.__modifierCapsLock = Boolean(options.modifierCapsLock);
}

function __getModifierState(event, key) {
    return ({
        Control: event.ctrlKey,
        Shift: event.shiftKey,
        Alt: event.altKey,
        Meta: event.metaKey,
        AltGraph: event.__modifierAltGraph,
        CapsLock: event.__modifierCapsLock,
    })[String(key)] ?? false;
}

class MouseEvent extends UIEvent {
    constructor(type, options = {}) {
        super(type, options);
        this.screenX = Number(options.screenX ?? 0);
        this.screenY = Number(options.screenY ?? 0);
        this.clientX = Number(options.clientX ?? 0);
        this.clientY = Number(options.clientY ?? 0);
        this.button = Number(options.button ?? 0);
        this.buttons = Number(options.buttons ?? 0);
        this.relatedTarget = options.relatedTarget === undefined ? null : options.relatedTarget;
        this.movementX = Number(options.movementX ?? 0);
        this.movementY = Number(options.movementY ?? 0);
        __initializeModifiers(this, options);
    }
    get x() { return this.clientX; }
    get y() { return this.clientY; }
    get pageX() { return this.clientX; }
    get pageY() { return this.clientY; }
    get offsetX() { return this.clientX; }
    get offsetY() { return this.clientY; }
    getModifierState(key) { return __getModifierState(this, key); }
    initMouseEvent(type, bubbles = false, cancelable = false, view = null, detail = 0,
                   screenX = 0, screenY = 0, clientX = 0, clientY = 0,
                   ctrlKey = false, altKey = false, shiftKey = false, metaKey = false,
                   button = 0, relatedTarget = null) {
        if (this.__dispatching) return;
        this.initUIEvent(type, bubbles, cancelable, view, detail);
        Object.assign(this, {
            screenX: Number(screenX), screenY: Number(screenY),
            clientX: Number(clientX), clientY: Number(clientY),
            ctrlKey: Boolean(ctrlKey), altKey: Boolean(altKey),
            shiftKey: Boolean(shiftKey), metaKey: Boolean(metaKey),
            button: Number(button), buttons: 0, relatedTarget,
        });
    }
}

class KeyboardEvent extends UIEvent {
    constructor(type, options = {}) {
        super(type, options);
        this.key = String(options.key ?? "");
        this.code = String(options.code ?? "");
        this.location = Number(options.location ?? 0);
        this.repeat = Boolean(options.repeat);
        this.isComposing = Boolean(options.isComposing);
        this.charCode = 0;
        this.keyCode = 0;
        this.which = 0;
        __initializeModifiers(this, options);
    }
    getModifierState(key) { return __getModifierState(this, key); }
}
KeyboardEvent.DOM_KEY_LOCATION_STANDARD = 0;
KeyboardEvent.DOM_KEY_LOCATION_LEFT = 1;
KeyboardEvent.DOM_KEY_LOCATION_RIGHT = 2;
KeyboardEvent.DOM_KEY_LOCATION_NUMPAD = 3;

class MessageEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        this.data = options.data === undefined ? null : options.data;
        this.origin = String(options.origin ?? "");
        this.lastEventId = String(options.lastEventId ?? "");
        this.source = options.source === undefined ? null : options.source;
        this.ports = Object.freeze(Array.from(options.ports ?? []));
    }
}

class StorageEvent extends Event {
    constructor(type, options = {}) {
        if (arguments.length === 0) throw new TypeError("StorageEvent type is required");
        super(type, options);
        this.key = options.key == null ? null : String(options.key);
        this.oldValue = options.oldValue == null ? null : String(options.oldValue);
        this.newValue = options.newValue == null ? null : String(options.newValue);
        this.url = options.url === undefined ? "" : String(options.url);
        this.storageArea = options.storageArea === undefined ? null : options.storageArea;
    }
    initStorageEvent(type, bubbles = false, cancelable = false, key = null,
                     oldValue = null, newValue = null, url = "", storageArea = null) {
        if (arguments.length === 0) throw new TypeError("StorageEvent type is required");
        if (this.__dispatching) return;
        this.initEvent(type, bubbles, cancelable);
        this.key = key == null ? null : String(key);
        this.oldValue = oldValue == null ? null : String(oldValue);
        this.newValue = newValue == null ? null : String(newValue);
        this.url = String(url);
        this.storageArea = storageArea;
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

function __cloneMessageValue(value, transferred, seen = new Map()) {
    if (value === null || typeof value !== "object") return value;
    if (transferred.has(value)) return transferred.get(value);
    if (seen.has(value)) return seen.get(value);
    if (value instanceof ArrayBuffer) return value.slice(0);
    if (ArrayBuffer.isView(value)) {
        const buffer = __cloneMessageValue(value.buffer, transferred, seen);
        return value instanceof DataView
            ? new DataView(buffer, value.byteOffset, value.byteLength)
            : new value.constructor(buffer, value.byteOffset, value.length);
    }
    if (Array.isArray(value)) {
        const clone = [];
        seen.set(value, clone);
        for (const item of value) clone.push(__cloneMessageValue(item, transferred, seen));
        return clone;
    }
    const clone = {};
    seen.set(value, clone);
    for (const key of Object.keys(value)) clone[key] = __cloneMessageValue(value[key], transferred, seen);
    return clone;
}

function __prepareMessage(message, transferOrOptions) {
    const transfer = transferOrOptions == null
        ? []
        : Array.from(Array.isArray(transferOrOptions) ? transferOrOptions : transferOrOptions.transfer ?? []);
    const transferred = new Map();
    for (const item of transfer) {
        if (transferred.has(item)) throw new DOMException("Transfer list contains a duplicate", "DataCloneError");
        if (item instanceof ArrayBuffer) {
            transferred.set(item, item.slice(0));
        } else {
            throw new DOMException("Object is not transferable", "DataCloneError");
        }
    }
    const cloned = __cloneMessageValue(message, transferred);
    for (const item of transfer) item.transfer();
    return cloned;
}

class MessagePort extends EventTarget {
    constructor() {
        super();
        this.__entangled = null;
        this.__queue = [];
        this.__started = false;
        this.__closed = false;
        this.__scheduled = false;
        this.__onmessage = null;
        this.onmessageerror = null;
    }
    get onmessage() { return this.__onmessage; }
    set onmessage(callback) {
        this.__onmessage = callback == null ? null : callback;
        if (this.__onmessage !== null) this.start();
    }
    postMessage(message, transferOrOptions = []) {
        if (arguments.length === 0) throw new TypeError("message is required");
        const cloned = __prepareMessage(message, transferOrOptions);
        const target = this.__entangled;
        if (!target || this.__closed || target.__closed) return;
        target.__queue.push(cloned);
        target.__schedule();
    }
    start() {
        if (this.__closed) return;
        this.__started = true;
        this.__schedule();
    }
    close() {
        this.__closed = true;
        this.__queue.length = 0;
        this.__entangled = null;
    }
    __schedule() {
        if (!this.__started || this.__scheduled || this.__closed || this.__queue.length === 0) return;
        this.__scheduled = true;
        setTimeout(() => {
            this.__scheduled = false;
            if (!this.__started || this.__closed) return;
            const hasMessage = this.__queue.length !== 0;
            const data = this.__queue.shift();
            if (hasMessage) {
                const event = new MessageEvent("message", { data });
                event.isTrusted = true;
                this.dispatchEvent(event);
            }
            this.__schedule();
        }, 0);
    }
}

class MessageChannel {
    constructor() {
        const port1 = new MessagePort();
        const port2 = new MessagePort();
        port1.__entangled = port2;
        port2.__entangled = port1;
        Object.defineProperties(this, {
            port1: { value: port1, enumerable: true },
            port2: { value: port2, enumerable: true },
        });
    }
}

function postMessage(message, targetOrigin = "/", transfer = []) {
    const cloned = __prepareMessage(message, transfer);
    targetOrigin = String(targetOrigin);
    if (targetOrigin !== "*" && targetOrigin !== "/" && targetOrigin !== location.origin) return;
    setTimeout(() => {
        const event = new MessageEvent("message", {
            data: cloned,
            origin: location.origin,
            source: window,
        });
        event.isTrusted = true;
        window.dispatchEvent(event);
    }, 0);
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
        return this.__sheet.__rules().length;
    }
    item(index) {
        __requireCssRuleList(this);
        if (arguments.length === 0) throw new TypeError("CSSRuleList.item requires an index");
        return this.__sheet.__rules()[Number(index)] ?? null;
    }
    [Symbol.iterator]() { return this.__sheet.__rules()[Symbol.iterator](); }
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

function __removeNode(node) {
    if (node.parentNode) node.parentNode.removeChild(node);
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
    querySelector(selector) { return __callHost("querySelector", this, selector); }
    querySelectorAll(selector) { return new NodeList(__callHost("querySelectorAll", this, selector)); }
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

function __insertAdjacentNode(element, position, node) {
    if (position === "beforebegin") {
        if (!element.parentNode) return null;
        element.parentNode.insertBefore(node, element);
    } else if (position === "afterbegin") {
        element.insertBefore(node, element.firstChild);
    } else if (position === "beforeend") {
        element.appendChild(node);
    } else if (position === "afterend") {
        if (!element.parentNode) return null;
        element.parentNode.insertBefore(node, element.nextSibling);
    } else {
        throw new DOMException("Invalid position", "SyntaxError");
    }
    return node;
}

class Element extends Node {
    get tagName() { return __callHost("tagName", this); }
    get localName() { return __callHost("localName", this); }
    get namespaceURI() { return __callHost("namespaceURI", this); }
    get prefix() { return __callHost("prefix", this); }
    get children() { return new HTMLCollection(() => [...this.childNodes].filter(node => node instanceof Element)); }
    get childElementCount() { return this.children.length; }
    get firstElementChild() { return this.children.item(0); }
    get lastElementChild() { return this.children.item(this.children.length - 1); }
    get previousElementSibling() {
        let sibling = this.previousSibling;
        while (sibling !== null && !(sibling instanceof Element)) sibling = sibling.previousSibling;
        return sibling;
    }
    get nextElementSibling() {
        let sibling = this.nextSibling;
        while (sibling !== null && !(sibling instanceof Element)) sibling = sibling.nextSibling;
        return sibling;
    }
    get attributes() {
        let attributes = __attributeMaps.get(this);
        if (!attributes) {
            attributes = new NamedNodeMap(this);
            __attributeMaps.set(this, attributes);
        }
        return attributes;
    }
    get id() { return __callHost("getAttributeOrEmpty", this, "id"); }
    set id(value) { __callHost("setAttribute", this, "id", value); }
    get className() { return __callHost("getAttributeOrEmpty", this, "class"); }
    set className(value) { __callHost("setAttribute", this, "class", value); }
    get classList() {
        let list = __classLists.get(this);
        if (!list) { list = new DOMTokenList(this); __classLists.set(this, list); }
        return list;
    }
    set classList(value) { this.classList.value = value; }
    get innerHTML() { return __callHost("innerHTML", this); }
    set innerHTML(value) { __callHost("setInnerHTML", this, value); }
    get style() { return __styleDeclarationProxy(__callHost("style", this)); }
    set style(value) { this.style.cssText = value; }
    get clientWidth() { return __callHost("clientWidth", this); }
    get clientHeight() { return __callHost("clientHeight", this); }
    get offsetWidth() { return __callHost("offsetWidth", this); }
    get offsetHeight() { return __callHost("offsetHeight", this); }
    getBoundingClientRect() {
        const rect = __callHost("boundingRect", this);
        return new DOMRect(rect[0], rect[1], rect[2], rect[3]);
    }
    getClientRects() {
        const rect = this.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 ? [rect] : [];
    }
    getAttribute(name) { return __callHost("getAttribute", this, name); }
    setAttribute(name, value) {
        name = String(name);
        if (name === "") throw new DOMException("attribute name cannot be empty", "InvalidCharacterError");
        __callHost("setAttribute", this, name, value);
    }
    removeAttribute(name) { __callHost("removeAttribute", this, name); }
    hasAttribute(name) { return this.getAttribute(name) !== null; }
    getAttributeNames() { return __attributeRecords(this).map(attribute => attribute.name); }
    toggleAttribute(name, force = undefined) {
        name = String(name);
        if (name === "") throw new DOMException("attribute name cannot be empty", "InvalidCharacterError");
        const present = this.hasAttribute(name);
        if (present && (arguments.length < 2 || !Boolean(force))) {
            this.removeAttribute(name);
            return false;
        }
        if (!present && (arguments.length < 2 || Boolean(force))) this.setAttribute(name, "");
        return true;
    }
    append(...nodes) { __appendNodes(this, nodes); }
    prepend(...nodes) { __prependNodes(this, nodes); }
    replaceChildren(...nodes) { __replaceChildren(this, nodes); }
    remove() { __removeNode(this); }
    insertAdjacentElement(position, element) {
        if (!(element instanceof Element)) throw new TypeError("element must be an Element");
        position = String(position).toLowerCase();
        return __insertAdjacentNode(this, position, element);
    }
    insertAdjacentText(position, data) {
        __insertAdjacentNode(this, String(position).toLowerCase(), document.createTextNode(String(data)));
    }
    insertAdjacentHTML(position, text) {
        position = String(position).toLowerCase();
        const outside = position === "beforebegin" || position === "afterend";
        if (outside && !this.parentNode) throw new DOMException("The element has no parent", "NoModificationAllowedError");
        if (!["beforebegin", "afterbegin", "beforeend", "afterend"].includes(position)) {
            throw new DOMException("Invalid position", "SyntaxError");
        }
        const container = document.createElement(outside && this.parentElement ? this.parentElement.localName : this.localName);
        container.innerHTML = String(text);
        const nodes = Array.from(container.childNodes);
        if (position === "afterbegin" || position === "afterend") nodes.reverse();
        for (const node of nodes) __insertAdjacentNode(this, position, node);
    }
    getElementsByTagName(name) {
        return new HTMLCollection(() => __callHost("getElementsByTagName", this, name));
    }
    getElementsByClassName(names) {
        return new HTMLCollection(() => __callHost("getElementsByClassName", this, names));
    }
    querySelector(selector) { return __callHost("querySelector", this, selector); }
    querySelectorAll(selector) { return new NodeList(__callHost("querySelectorAll", this, selector)); }
    matches(selector) { return __callHost("matches", this, selector); }
    closest(selector) {
        let element = this;
        while (element) {
            if (element.matches(selector)) return element;
            element = element.parentElement;
        }
        return null;
    }
    scrollIntoView() {}
    focus() {
        const previous = __activeElement;
        if (previous === this) return;
        if (previous) {
            previous.dispatchEvent(new Event("blur"));
            previous.dispatchEvent(new Event("focusout", {bubbles: true}));
        }
        __activeElement = this;
        this.dispatchEvent(new Event("focus"));
        this.dispatchEvent(new Event("focusin", {bubbles: true}));
    }
    blur() {
        if (__activeElement !== this) return;
        __activeElement = null;
        this.dispatchEvent(new Event("blur"));
        this.dispatchEvent(new Event("focusout", {bubbles: true}));
    }
    click() { this.dispatchEvent(new Event("click", { bubbles: true, cancelable: true })); }
}

function __reflectedString(element, name) {
    return element.getAttribute(name) || "";
}

function __setReflectedBoolean(element, name, value) {
    if (Boolean(value)) element.setAttribute(name, "");
    else element.removeAttribute(name);
}

class SVGElement extends Element {}

class HTMLElement extends Element {
    get innerText() { return this.textContent; }
    set innerText(value) { this.textContent = String(value); }
    get title() { return __reflectedString(this, "title"); }
    set title(value) { this.setAttribute("title", value); }
    get lang() { return __reflectedString(this, "lang"); }
    set lang(value) { this.setAttribute("lang", value); }
    get dir() {
        const value = __asciiLowercase(__reflectedString(this, "dir"));
        return value === "ltr" || value === "rtl" || value === "auto" ? value : "";
    }
    set dir(value) { this.setAttribute("dir", value); }
    get autofocus() { return this.hasAttribute("autofocus"); }
    set autofocus(value) { __setReflectedBoolean(this, "autofocus", value); }
    get hidden() { return this.hasAttribute("hidden"); }
    set hidden(value) { __setReflectedBoolean(this, "hidden", value); }
    get accessKey() { return __reflectedString(this, "accesskey"); }
    set accessKey(value) { this.setAttribute("accesskey", value); }
    get tabIndex() {
        const value = this.getAttribute("tabindex");
        if (value === null) return -1;
        const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
        if (!match) return 0;
        const parsed = Number(match[1]);
        if (parsed === 0) return 0;
        return parsed >= -2147483648 && parsed <= 2147483647 ? parsed : 0;
    }
    set tabIndex(value) {
        let number = Number(value);
        if (!Number.isFinite(number) || number === 0) number = 0;
        else number = Math.trunc(number);
        number = ((number % 4294967296) + 4294967296) % 4294967296;
        if (number >= 2147483648) number -= 4294967296;
        this.setAttribute("tabindex", String(number));
    }
}
class HTMLAnchorElement extends HTMLElement {
    get href() { return this.hasAttribute("href") ? __callHost("elementUrl", this, "href") : ""; }
    set href(value) {
        value = __toUSVString(value);
        const queryStart = value.indexOf("?");
        if (queryStart !== -1 && __asciiLowercase(document.characterSet) !== "utf-8") {
            const fragmentStart = value.indexOf(String.fromCharCode(35), queryStart);
            const queryEnd = fragmentStart === -1 ? value.length : fragmentStart;
            value = value.slice(0, queryStart + 1)
                + __legacyQueryEncode(document.characterSet, value.slice(queryStart + 1, queryEnd))
                + value.slice(queryEnd);
        }
        this.setAttribute("href", value);
    }
    get origin() { return __callHost("elementUrl", this, "origin"); }
    get protocol() { return new URL(this.href).protocol; }
    set protocol(value) { this.__setUrlComponent("protocol", value); }
    get username() { return new URL(this.href).username; }
    set username(value) { this.__setUrlComponent("username", value); }
    get password() { return new URL(this.href).password; }
    set password(value) { this.__setUrlComponent("password", value); }
    get host() { return new URL(this.href).host; }
    set host(value) { this.__setUrlComponent("host", value); }
    get hostname() { return new URL(this.href).hostname; }
    set hostname(value) { this.__setUrlComponent("hostname", value); }
    get port() { return new URL(this.href).port; }
    set port(value) { this.__setUrlComponent("port", value); }
    get pathname() { return new URL(this.href).pathname; }
    set pathname(value) { this.__setUrlComponent("pathname", value); }
    get search() { return new URL(this.href).search; }
    set search(value) { this.__setUrlComponent("search", value); }
    get hash() { return new URL(this.href).hash; }
    set hash(value) { this.__setUrlComponent("hash", value); }
    toString() { return this.href; }
    __setUrlComponent(component, value) {
        const url = new URL(this.href);
        url[component] = value;
        this.href = url.href;
    }
}

const __legacyEncodingBlocks = new Map();
function __legacyQueryEncode(label, input) {
    if (__asciiLowercase(label) === "iso-2022-jp") {
        return __callHost("legacyQueryEncode", window, label, input);
    }
    let blocks = __legacyEncodingBlocks.get(label);
    if (!blocks) {
        blocks = new Map();
        __legacyEncodingBlocks.set(label, blocks);
    }
    let output = "";
    for (const character of input) {
        const codePoint = character.codePointAt(0);
        if (codePoint < 128) {
            output += character;
            continue;
        }
        const blockStart = codePoint - (codePoint % 256);
        let block = blocks.get(blockStart);
        if (!block) {
            block = JSON.parse(__callHost("legacyQueryEncodeBlock", window, label, blockStart));
            blocks.set(blockStart, block);
        }
        output += block[codePoint - blockStart];
    }
    return output;
}
class HTMLBaseElement extends HTMLElement {
    get href() { return __callHost("elementUrl", this, "href"); }
    set href(value) { this.setAttribute("href", value); }
}

function __illegalHtmlElementConstructor() {
    throw new TypeError("Illegal constructor");
}

function __defineStringReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase()] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() { return this.getAttribute(attribute) ?? ""; },
            set(value) { this.setAttribute(attribute, String(value)); },
        });
    }
}

function __defineBooleanReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase()] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() { return this.hasAttribute(attribute); },
            set(value) { __setReflectedBoolean(this, attribute, value); },
        });
    }
}

function __parseNonnegativeInteger(value) {
    const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
    if (!match) return null;
    const number = Number(match[1]);
    if (number === 0) return 0;
    return Number.isSafeInteger(number) && number > 0 && number <= 2147483647 ? number : null;
}

function __defineUnsignedReflections(prototype, definitions) {
    for (const [property, attribute, defaultValue = 0] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const value = this.getAttribute(attribute);
                return value === null ? defaultValue : (__parseNonnegativeInteger(value) ?? defaultValue);
            },
            set(value) {
                const converted = Number(value) >>> 0;
                this.setAttribute(attribute, String(converted <= 2147483647 ? converted : defaultValue));
            },
        });
    }
}

function __defineUrlReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase()] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() { return this.hasAttribute(attribute) ? __callHost("elementUrl", this, attribute) : ""; },
            set(value) { this.setAttribute(attribute, __toUSVString(value)); },
        });
    }
}

function __defineActionUrlReflection(prototype, property, attribute) {
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() { return __callHost("elementUrl", this, attribute); },
        set(value) { this.setAttribute(attribute, __toUSVString(value)); },
    });
}

function __defineEnumReflection(prototype, property, attribute, keywords, defaultValue, invalidValue, aliases = {}, nullable = false) {
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() {
            const raw = this.getAttribute(attribute);
            if (raw === null) return defaultValue;
            const value = __asciiLowercase(raw);
            if (Object.prototype.hasOwnProperty.call(aliases, value)) return aliases[value];
            return keywords.includes(value) ? value : invalidValue;
        },
        set(value) {
            if (nullable && value == null) this.removeAttribute(attribute);
            else this.setAttribute(attribute, String(value));
        },
    });
}

function __asciiLowercase(value) {
    return String(value).replace(/[A-Z]/g, character => String.fromCharCode(character.charCodeAt(0) + 32));
}

function __defineNullAsEmptyStringReflection(prototype, property, attribute) {
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() { return this.getAttribute(attribute) ?? ""; },
        set(value) { this.setAttribute(attribute, value === null ? "" : String(value)); },
    });
}

function __defineTokenListReflection(prototype, property, attribute) {
    const lists = new WeakMap();
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() {
            let list = lists.get(this);
            if (!list) { list = new DOMTokenList(this, attribute); lists.set(this, list); }
            return list;
        },
    });
}

function __parseInteger(value) {
    const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
    if (!match) return null;
    const number = Number(match[1]);
    if (number === 0) return 0;
    return Number.isSafeInteger(number) && number >= -2147483648 && number <= 2147483647 ? number : null;
}

function __toLong(value) {
    let number = Number(value);
    if (!Number.isFinite(number) || number === 0) return 0;
    number = Math.trunc(number);
    number = ((number % 4294967296) + 4294967296) % 4294967296;
    return number >= 2147483648 ? number - 4294967296 : number;
}

function __defineLongReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase(), defaultValue = 0, nonnegative = false] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const parsed = __parseInteger(this.getAttribute(attribute) ?? "");
                return parsed === null || (nonnegative && parsed < 0) ? defaultValue : parsed;
            },
            set(value) {
                const converted = __toLong(value);
                if (nonnegative && converted < 0) throw new DOMException("value must be non-negative", "IndexSizeError");
                this.setAttribute(attribute, String(converted));
            },
        });
    }
}

function __definePositiveUnsignedReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase(), defaultValue = 1, fallback = false] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const parsed = __parseNonnegativeInteger(this.getAttribute(attribute) ?? "");
                return parsed === null || parsed === 0 ? defaultValue : parsed;
            },
            set(value) {
                const converted = Number(value) >>> 0;
                if (!fallback && converted === 0) throw new DOMException("value must be greater than zero", "IndexSizeError");
                this.setAttribute(attribute, String(converted > 0 && converted <= 2147483647 ? converted : defaultValue));
            },
        });
    }
}

function __defineClampedUnsignedReflections(prototype, definitions) {
    for (const [property, attribute, defaultValue, minimum, maximum] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(this.getAttribute(attribute) ?? "");
                const parsed = match ? Number(match[1]) : NaN;
                if (!Number.isSafeInteger(parsed) || parsed < 0) return defaultValue;
                return Math.min(maximum, Math.max(minimum, parsed));
            },
            set(value) {
                const converted = Number(value) >>> 0;
                this.setAttribute(attribute, String(converted <= 2147483647 ? converted : defaultValue));
            },
        });
    }
}

function __parseFloatingPoint(value) {
    const match = /^[\t\n\f\r ]*([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)/.exec(value);
    if (!match) return null;
    const number = Number(match[1]);
    return Number.isFinite(number) ? (number === 0 ? 0 : number) : null;
}

function __defineDoubleReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase(), defaultValue = 0, positive = false] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const parsed = __parseFloatingPoint(this.getAttribute(attribute) ?? "");
                return parsed === null || (positive && parsed <= 0) ? defaultValue : parsed;
            },
            set(value) {
                const number = Number(value);
                if (!Number.isFinite(number)) throw new DOMException("value must be finite", "NotSupportedError");
                if (!positive || number > 0) this.setAttribute(attribute, String(number === 0 ? 0 : number));
            },
        });
    }
}

function __defineSettableTokenListReflection(prototype, property, attribute) {
    __defineTokenListReflection(prototype, property, attribute);
    const descriptor = Object.getOwnPropertyDescriptor(prototype, property);
    Object.defineProperty(prototype, property, {
        ...descriptor,
        set(value) { this.setAttribute(attribute, String(value)); },
    });
}

function __defineNonceReflection(prototype) {
    const values = new WeakMap();
    Object.defineProperty(prototype, "nonce", {
        configurable: true, enumerable: true,
        get() { return values.has(this) ? values.get(this) : (this.getAttribute("nonce") ?? ""); },
        set(value) { values.set(this, String(value)); },
    });
}

class HTMLPictureElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLImageElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }

function __decodeDataUrlDocument(href) {
    const comma = href.indexOf(",");
    if (comma === -1) return "";
    const metadata = href.slice(5, comma);
    const payload = href.slice(comma + 1);
    const label = /(?:^|;)charset=([^;]*)/i.exec(metadata)?.[1] ?? "US-ASCII";
    let byteString;
    if (/(?:^|;)base64(?:;|$)/i.test(metadata)) byteString = atob(payload);
    else byteString = unescape(payload);
    const bytes = new Uint8Array(byteString.length);
    for (let index = 0; index < byteString.length; index++) bytes[index] = byteString.charCodeAt(index);
    return new TextDecoder(label).decode(bytes);
}

function __runInChildWindow(context, source) {
    return Function("window", "document", "location", "parent", "self", "top", source)(
        context.window,
        context.document,
        context.window.location,
        window,
        context.window,
        window,
    );
}

function __childConstructor(context, creator) {
    return function(...arguments_) { return creator(context.document, ...arguments_); };
}

class HTMLIFrameElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get contentWindow() {
        let context = __iframeBrowsingContexts.get(this);
        if (!context) {
            let location = new URL("about:blank");
            const childWindow = new EventTarget();
            context = {
                document: new DOMParser().parseFromString("", "text/html"),
                window: childWindow,
                navigate(url) {
                    location = new URL(url, document.URL);
                    const source = location.protocol === "data:"
                        ? __decodeDataUrlDocument(location.href)
                        : "";
                    context.document = new DOMParser().parseFromString(source, "text/html");
                    Object.defineProperty(context.document, "__URL", {
                        value: location.href, configurable: true,
                    });
                    for (const script of context.document.querySelectorAll("script")) {
                        __runInChildWindow(context, script.textContent);
                    }
                    const onload = context.document.body?.getAttribute("onload");
                    if (onload) __runInChildWindow(context, onload);
                },
            };
            Object.defineProperties(childWindow, {
                location: {
                    get() { return location; },
                    set(value) { context.navigate(value); },
                    enumerable: true,
                },
                document: { get() { return context.document; }, enumerable: true },
                frameElement: { value: this, enumerable: true },
                parent: { value: window, enumerable: true },
                top: { value: window, enumerable: true },
            });
            childWindow.window = childWindow;
            childWindow.self = childWindow;
            childWindow.Comment = __childConstructor(context, (doc, data = "") => doc.createComment(String(data)));
            childWindow.Text = __childConstructor(context, (doc, data = "") => doc.createTextNode(String(data)));
            childWindow.DocumentFragment = __childConstructor(context, doc => doc.createDocumentFragment());
            childWindow.postMessage = (message, targetOrigin = "*", transfer = []) => {
                const cloned = __prepareMessage(message, transfer);
                targetOrigin = String(targetOrigin);
                if (targetOrigin !== "*" && targetOrigin !== "/" && targetOrigin !== location.origin) return;
                setTimeout(() => {
                    const event = new MessageEvent("message", {
                        data: cloned,
                        origin: document.location?.origin ?? location.origin,
                        source: window,
                    });
                    event.isTrusted = true;
                    childWindow.dispatchEvent(event);
                }, 0);
            };
            __iframeBrowsingContexts.set(this, context);
        }
        return context.window;
    }
    get contentDocument() {
        this.contentWindow;
        return __iframeBrowsingContexts.get(this).document;
    }
    __connected() {
        if (__iframeBrowsingContexts.get(this)?.connected) return;
        this.contentWindow;
        const context = __iframeBrowsingContexts.get(this);
        context.connected = true;
        context.navigate(this.getAttribute("srcdoc") !== null
            ? "data:text/html;charset=utf-8," + escape(this.getAttribute("srcdoc"))
            : (this.getAttribute("src") || "about:blank"));
        setTimeout(() => this.dispatchEvent(new Event("load")), 0);
    }
    __navigate(url) {
        this.contentWindow;
        const context = __iframeBrowsingContexts.get(this);
        context.navigate(url);
        setTimeout(() => this.dispatchEvent(new Event("load")), 0);
    }
}
class HTMLEmbedElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLObjectElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLParamElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMediaElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLVideoElement extends HTMLMediaElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLAudioElement extends HTMLMediaElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLSourceElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTrackElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLCanvasElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMapElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLAreaElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFormElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    submit() {
        if (this.method !== "get") {
            throw new DOMException("Only GET form submission is implemented", "NotSupportedError");
        }
        const charset = this.acceptCharset || document.characterSet;
        const pairs = [];
        for (const control of this.querySelectorAll("input,button,select,textarea")) {
            if (!control.name || control.disabled) continue;
            if (control instanceof HTMLInputElement &&
                ["button", "reset", "submit", "image", "file"].includes(control.type)) continue;
            pairs.push(__callHost("formUrlEncode", window, charset, control.name) + "=" +
                __callHost("formUrlEncode", window, charset, control.value ?? ""));
        }
        const destination = new URL(this.action || document.URL, document.URL);
        destination.search = pairs.join("&");
        const target = this.target;
        let frame = null;
        if (target) {
            for (const candidate of document.getElementsByTagName("iframe")) {
                if (candidate.name === target || candidate.id === target) {
                    frame = candidate;
                    break;
                }
            }
        }
        if (frame) frame.__navigate(destination.href);
        else location.href = destination.href;
    }
}
class HTMLFieldSetElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLegendElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLabelElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLInputElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get value() {
        return __inputValues.has(this) ? __inputValues.get(this) : (this.getAttribute("value") ?? "");
    }
    set value(value) { __inputValues.set(this, String(value)); }
}
class HTMLButtonElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLSelectElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDataListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLOptGroupElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLOptionElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTextAreaElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get value() { return __inputValues.has(this) ? __inputValues.get(this) : this.textContent; }
    set value(value) { __inputValues.set(this, String(value)); }
}
class HTMLOutputElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLProgressElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMeterElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableCaptionElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableColElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableSectionElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableRowElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableCellElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHeadElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTitleElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLinkElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get sheet() { return __styleSheetForOwner(this); }
    get disabled() { return this.sheet?.disabled ?? false; }
    set disabled(value) {
        const sheet = this.sheet;
        if (sheet !== null) sheet.disabled = Boolean(value);
    }
}
class HTMLMetaElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLStyleElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get sheet() { return __styleSheetForOwner(this); }
    get disabled() { return this.sheet?.disabled ?? false; }
    set disabled(value) {
        const sheet = this.sheet;
        if (sheet !== null) sheet.disabled = Boolean(value);
    }
}
class HTMLParagraphElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHRElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLPreElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLQuoteElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLOListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLUListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLIElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDivElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDataElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTimeElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLBRElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLBodyElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHeadingElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHtmlElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLScriptElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTemplateElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get content() {
        let content = __templateContents.get(this);
        if (!content) {
            content = this.ownerDocument.createDocumentFragment();
            while (this.firstChild) content.appendChild(this.firstChild);
            __templateContents.set(this, content);
        }
        return content;
    }
}
class HTMLSlotElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLModElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDetailsElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMenuElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDialogElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMarqueeElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFrameSetElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFrameElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDirectoryElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFontElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }

__defineStringReflections(HTMLImageElement.prototype, [
    ["alt"], ["srcset"], ["useMap", "usemap"], ["name"], ["align"], ["border"],
]);
__defineUrlReflections(HTMLImageElement.prototype, [["src"], ["lowsrc"], ["longDesc", "longdesc"]]);
__defineBooleanReflections(HTMLImageElement.prototype, [["isMap", "ismap"]]);
__defineUnsignedReflections(HTMLImageElement.prototype, [
    ["width", "width", 0], ["height", "height", 0], ["hspace", "hspace", 0], ["vspace", "vspace", 0],
]);
__defineEnumReflection(HTMLImageElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineEnumReflection(HTMLImageElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineEnumReflection(HTMLImageElement.prototype, "decoding", "decoding", ["async", "sync", "auto"], "auto", "auto");
__defineNullAsEmptyStringReflection(HTMLImageElement.prototype, "border", "border");

__defineStringReflections(HTMLIFrameElement.prototype, [
    ["srcdoc"], ["name"], ["width"], ["height"], ["align"], ["scrolling"],
    ["frameBorder", "frameborder"], ["marginHeight", "marginheight"], ["marginWidth", "marginwidth"],
]);
__defineUrlReflections(HTMLIFrameElement.prototype, [["src"], ["longDesc", "longdesc"]]);
__defineBooleanReflections(HTMLIFrameElement.prototype, [["allowFullscreen", "allowfullscreen"]]);
__defineEnumReflection(HTMLIFrameElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineNullAsEmptyStringReflection(HTMLIFrameElement.prototype, "marginHeight", "marginheight");
__defineNullAsEmptyStringReflection(HTMLIFrameElement.prototype, "marginWidth", "marginwidth");

__defineStringReflections(HTMLEmbedElement.prototype, [["type"], ["width"], ["height"], ["align"], ["name"]]);
__defineUrlReflections(HTMLEmbedElement.prototype, [["src"]]);
__defineStringReflections(HTMLObjectElement.prototype, [
    ["type"], ["name"], ["useMap", "usemap"], ["width"], ["height"], ["align"],
    ["archive"], ["code"], ["standby"], ["codeType", "codetype"], ["border"],
]);
__defineUrlReflections(HTMLObjectElement.prototype, [["data"], ["codeBase", "codebase"]]);
__defineBooleanReflections(HTMLObjectElement.prototype, [["declare"]]);
__defineUnsignedReflections(HTMLObjectElement.prototype, [["hspace", "hspace", 0], ["vspace", "vspace", 0]]);
__defineNullAsEmptyStringReflection(HTMLObjectElement.prototype, "border", "border");
__defineStringReflections(HTMLParamElement.prototype, [["name"], ["value"], ["type"], ["valueType", "valuetype"]]);

__defineUrlReflections(HTMLMediaElement.prototype, [["src"]]);
__defineBooleanReflections(HTMLMediaElement.prototype, [["autoplay"], ["loop"], ["controls"], ["defaultMuted", "muted"]]);
__defineEnumReflection(HTMLMediaElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineEnumReflection(HTMLMediaElement.prototype, "preload", "preload", ["none", "metadata", "auto"], "metadata", "metadata", { "": "auto" });
__defineEnumReflection(HTMLMediaElement.prototype, "loading", "loading", ["lazy", "eager"], "eager", "eager");
__defineUnsignedReflections(HTMLVideoElement.prototype, [["width", "width", 0], ["height", "height", 0]]);
__defineUrlReflections(HTMLVideoElement.prototype, [["poster"]]);
__defineBooleanReflections(HTMLVideoElement.prototype, [["playsInline", "playsinline"]]);
__defineTokenListReflection(HTMLVideoElement.prototype, "controlsList", "controlslist");

__defineStringReflections(HTMLSourceElement.prototype, [["type"], ["srcset"], ["sizes"], ["media"]]);
__defineUrlReflections(HTMLSourceElement.prototype, [["src"]]);
__defineStringReflections(HTMLTrackElement.prototype, [["srclang"], ["label"]]);
__defineUrlReflections(HTMLTrackElement.prototype, [["src"]]);
__defineBooleanReflections(HTMLTrackElement.prototype, [["default"]]);
__defineEnumReflection(HTMLTrackElement.prototype, "kind", "kind", ["subtitles", "captions", "descriptions", "chapters", "metadata"], "subtitles", "metadata");
__defineUnsignedReflections(HTMLCanvasElement.prototype, [["width", "width", 300], ["height", "height", 150]]);
__defineStringReflections(HTMLMapElement.prototype, [["name"]]);
__defineStringReflections(HTMLAreaElement.prototype, [["alt"], ["coords"], ["shape"], ["target"], ["download"], ["ping"], ["rel"], ["hreflang"], ["type"]]);
__defineUrlReflections(HTMLAreaElement.prototype, [["href"]]);
__defineBooleanReflections(HTMLAreaElement.prototype, [["noHref", "nohref"]]);
__defineTokenListReflection(HTMLAreaElement.prototype, "relList", "rel");
__defineEnumReflection(HTMLAreaElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");

__defineStringReflections(HTMLAnchorElement.prototype, [
    ["target"], ["download"], ["ping"], ["rel"], ["hreflang"], ["type"],
    ["coords"], ["charset"], ["name"], ["rev"], ["shape"],
]);
__defineTokenListReflection(HTMLAnchorElement.prototype, "relList", "rel");
__defineEnumReflection(HTMLAnchorElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineStringReflections(HTMLBaseElement.prototype, [["target"]]);

__defineStringReflections(HTMLFormElement.prototype, [["acceptCharset", "accept-charset"], ["name"], ["target"]]);
__defineActionUrlReflection(HTMLFormElement.prototype, "action", "action");
__defineBooleanReflections(HTMLFormElement.prototype, [["noValidate", "novalidate"]]);
__defineEnumReflection(HTMLFormElement.prototype, "autocomplete", "autocomplete", ["on", "off"], "on", "on");
__defineEnumReflection(HTMLFormElement.prototype, "enctype", "enctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "application/x-www-form-urlencoded", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLFormElement.prototype, "encoding", "enctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "application/x-www-form-urlencoded", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLFormElement.prototype, "method", "method", ["get", "post", "dialog"], "get", "get");
__defineStringReflections(HTMLFieldSetElement.prototype, [["name"]]);
__defineBooleanReflections(HTMLFieldSetElement.prototype, [["disabled"]]);
__defineStringReflections(HTMLLegendElement.prototype, [["align"]]);
__defineStringReflections(HTMLLabelElement.prototype, [["htmlFor", "for"]]);

__defineStringReflections(HTMLInputElement.prototype, [
    ["accept"], ["alt"], ["autocomplete"], ["dirName", "dirname"], ["max"], ["min"],
    ["name"], ["pattern"], ["placeholder"], ["step"], ["defaultValue", "value"],
    ["align"], ["useMap", "usemap"], ["formTarget", "formtarget"],
]);
__defineBooleanReflections(HTMLInputElement.prototype, [
    ["defaultChecked", "checked"], ["disabled"], ["formNoValidate", "formnovalidate"],
    ["multiple"], ["readOnly", "readonly"], ["required"],
]);
__defineUrlReflections(HTMLInputElement.prototype, [["src"]]);
__defineActionUrlReflection(HTMLInputElement.prototype, "formAction", "formaction");
__defineUnsignedReflections(HTMLInputElement.prototype, [["height", "height", 0], ["width", "width", 0]]);
__defineLongReflections(HTMLInputElement.prototype, [["maxLength", "maxlength", -1, true], ["minLength", "minlength", -1, true]]);
__definePositiveUnsignedReflections(HTMLInputElement.prototype, [["size", "size", 20, false]]);
__defineEnumReflection(HTMLInputElement.prototype, "formEnctype", "formenctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLInputElement.prototype, "formMethod", "formmethod", ["get", "post"], "", "get");
__defineEnumReflection(HTMLInputElement.prototype, "type", "type", ["hidden", "text", "search", "tel", "url", "email", "password", "date", "month", "week", "time", "datetime-local", "number", "range", "color", "checkbox", "radio", "file", "submit", "image", "reset", "button"], "text", "text");

__defineStringReflections(HTMLButtonElement.prototype, [["formTarget", "formtarget"], ["name"], ["value"]]);
__defineBooleanReflections(HTMLButtonElement.prototype, [["disabled"], ["formNoValidate", "formnovalidate"]]);
__defineActionUrlReflection(HTMLButtonElement.prototype, "formAction", "formaction");
__defineEnumReflection(HTMLButtonElement.prototype, "formEnctype", "formenctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLButtonElement.prototype, "formMethod", "formmethod", ["get", "post", "dialog"], "", "get");
__defineEnumReflection(HTMLButtonElement.prototype, "type", "type", ["submit", "reset", "button"], "submit", "submit");

__defineStringReflections(HTMLSelectElement.prototype, [["autocomplete"], ["name"]]);
__defineBooleanReflections(HTMLSelectElement.prototype, [["disabled"], ["multiple"], ["required"]]);
__defineUnsignedReflections(HTMLSelectElement.prototype, [["size", "size", 0]]);
__defineStringReflections(HTMLOptGroupElement.prototype, [["label"]]);
__defineBooleanReflections(HTMLOptGroupElement.prototype, [["disabled"]]);
__defineStringReflections(HTMLOptionElement.prototype, [["label"], ["value"]]);
__defineBooleanReflections(HTMLOptionElement.prototype, [["disabled"], ["defaultSelected", "selected"]]);
__defineStringReflections(HTMLTextAreaElement.prototype, [
    ["autocomplete"], ["dirName", "dirname"], ["name"], ["placeholder"], ["wrap"],
]);
__defineBooleanReflections(HTMLTextAreaElement.prototype, [["disabled"], ["readOnly", "readonly"], ["required"]]);
__defineLongReflections(HTMLTextAreaElement.prototype, [["maxLength", "maxlength", -1, true], ["minLength", "minlength", -1, true]]);
__definePositiveUnsignedReflections(HTMLTextAreaElement.prototype, [["cols", "cols", 20, true], ["rows", "rows", 2, true]]);
__defineSettableTokenListReflection(HTMLOutputElement.prototype, "htmlFor", "for");
__defineStringReflections(HTMLOutputElement.prototype, [["name"]]);
__defineDoubleReflections(HTMLProgressElement.prototype, [["max", "max", 1, true]]);
__defineDoubleReflections(HTMLMeterElement.prototype, [["value"], ["min"], ["max"], ["low"], ["high"], ["optimum"]]);

__defineStringReflections(HTMLTableElement.prototype, [
    ["align"], ["border"], ["frame"], ["rules"], ["summary"], ["width"],
    ["bgColor", "bgcolor"], ["cellPadding", "cellpadding"], ["cellSpacing", "cellspacing"],
]);
for (const [property, attribute] of [["bgColor", "bgcolor"], ["cellPadding", "cellpadding"], ["cellSpacing", "cellspacing"]]) {
    __defineNullAsEmptyStringReflection(HTMLTableElement.prototype, property, attribute);
}
__defineStringReflections(HTMLTableCaptionElement.prototype, [["align"]]);
__defineClampedUnsignedReflections(HTMLTableColElement.prototype, [["span", "span", 1, 1, 1000]]);
__defineStringReflections(HTMLTableColElement.prototype, [["align"], ["ch", "char"], ["chOff", "charoff"], ["vAlign", "valign"], ["width"]]);
__defineStringReflections(HTMLTableSectionElement.prototype, [["align"], ["ch", "char"], ["chOff", "charoff"], ["vAlign", "valign"]]);
__defineStringReflections(HTMLTableRowElement.prototype, [["align"], ["ch", "char"], ["chOff", "charoff"], ["vAlign", "valign"], ["bgColor", "bgcolor"]]);
__defineNullAsEmptyStringReflection(HTMLTableRowElement.prototype, "bgColor", "bgcolor");
__defineClampedUnsignedReflections(HTMLTableCellElement.prototype, [["colSpan", "colspan", 1, 1, 1000], ["rowSpan", "rowspan", 1, 0, 65534]]);
__defineStringReflections(HTMLTableCellElement.prototype, [
    ["headers"], ["abbr"], ["align"], ["axis"], ["height"], ["width"], ["ch", "char"],
    ["chOff", "charoff"], ["vAlign", "valign"], ["bgColor", "bgcolor"],
]);
__defineBooleanReflections(HTMLTableCellElement.prototype, [["noWrap", "nowrap"]]);
__defineEnumReflection(HTMLTableCellElement.prototype, "scope", "scope", ["row", "col", "rowgroup", "colgroup"], "", "");
__defineNullAsEmptyStringReflection(HTMLTableCellElement.prototype, "bgColor", "bgcolor");

__defineStringReflections(HTMLLinkElement.prototype, [
    ["rel"], ["media"], ["integrity"], ["hreflang"], ["type"], ["charset"], ["rev"], ["target"],
]);
__defineNonceReflection(HTMLLinkElement.prototype);
__defineUrlReflections(HTMLLinkElement.prototype, [["href"]]);
__defineTokenListReflection(HTMLLinkElement.prototype, "relList", "rel");
__defineSettableTokenListReflection(HTMLLinkElement.prototype, "sizes", "sizes");
__defineEnumReflection(HTMLLinkElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineEnumReflection(HTMLLinkElement.prototype, "as", "as", ["fetch", "audio", "document", "embed", "font", "image", "manifest", "object", "report", "script", "sharedworker", "style", "track", "video", "worker", "xslt"], "", "");
__defineEnumReflection(HTMLLinkElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineStringReflections(HTMLMetaElement.prototype, [["name"], ["httpEquiv", "http-equiv"], ["content"], ["media"], ["scheme"]]);
__defineStringReflections(HTMLStyleElement.prototype, [["media"], ["type"]]);
__defineNonceReflection(HTMLStyleElement.prototype);

__defineStringReflections(HTMLParagraphElement.prototype, [["align"]]);
__defineStringReflections(HTMLHRElement.prototype, [["align"], ["color"], ["size"], ["width"]]);
__defineBooleanReflections(HTMLHRElement.prototype, [["noShade", "noshade"]]);
__defineLongReflections(HTMLPreElement.prototype, [["width", "width", 0, false]]);
__defineUrlReflections(HTMLQuoteElement.prototype, [["cite"]]);
__defineBooleanReflections(HTMLOListElement.prototype, [["reversed"], ["compact"]]);
__defineLongReflections(HTMLOListElement.prototype, [["start", "start", 1, false]]);
__defineStringReflections(HTMLOListElement.prototype, [["type"]]);
__defineBooleanReflections(HTMLUListElement.prototype, [["compact"]]);
__defineStringReflections(HTMLUListElement.prototype, [["type"]]);
__defineLongReflections(HTMLLIElement.prototype, [["value", "value", 0, false]]);
__defineStringReflections(HTMLLIElement.prototype, [["type"]]);
__defineBooleanReflections(HTMLDListElement.prototype, [["compact"]]);
__defineStringReflections(HTMLDivElement.prototype, [["align"]]);
__defineStringReflections(HTMLDataElement.prototype, [["value"]]);
__defineStringReflections(HTMLTimeElement.prototype, [["dateTime", "datetime"]]);
__defineStringReflections(HTMLBRElement.prototype, [["clear"]]);

__defineStringReflections(HTMLBodyElement.prototype, [["text"], ["link"], ["vLink", "vlink"], ["aLink", "alink"], ["bgColor", "bgcolor"], ["background"]]);
for (const [property, attribute] of [["text", "text"], ["link", "link"], ["vLink", "vlink"], ["aLink", "alink"], ["bgColor", "bgcolor"]]) {
    __defineNullAsEmptyStringReflection(HTMLBodyElement.prototype, property, attribute);
}
__defineStringReflections(HTMLHeadingElement.prototype, [["align"]]);
for (const [property, attribute] of [["fgColor", "text"], ["linkColor", "link"], ["vlinkColor", "vlink"], ["alinkColor", "alink"], ["bgColor", "bgcolor"]]) {
    Object.defineProperty(Document.prototype, property, {
        configurable: true, enumerable: true,
        get() { return this.body?.getAttribute(attribute) ?? ""; },
        set(value) { if (this.body) this.body.setAttribute(attribute, value === null ? "" : String(value)); },
    });
}

__defineStringReflections(HTMLHtmlElement.prototype, [["version"]]);
__defineStringReflections(HTMLScriptElement.prototype, [["type"], ["charset"], ["integrity"], ["event"], ["htmlFor", "for"]]);
__defineUrlReflections(HTMLScriptElement.prototype, [["src"]]);
__defineBooleanReflections(HTMLScriptElement.prototype, [["noModule", "nomodule"], ["defer"]]);
__defineEnumReflection(HTMLScriptElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineStringReflections(HTMLSlotElement.prototype, [["name"]]);
__defineUrlReflections(HTMLModElement.prototype, [["cite"]]);
__defineStringReflections(HTMLModElement.prototype, [["dateTime", "datetime"]]);
__defineBooleanReflections(HTMLDetailsElement.prototype, [["open"]]);
__defineBooleanReflections(HTMLMenuElement.prototype, [["compact"]]);
__defineBooleanReflections(HTMLDialogElement.prototype, [["open"]]);
__defineEnumReflection(HTMLElement.prototype, "enterKeyHint", "enterkeyhint", ["enter", "done", "go", "next", "previous", "search", "send"], "", "");
__defineEnumReflection(HTMLElement.prototype, "inputMode", "inputmode", ["none", "text", "tel", "url", "email", "numeric", "decimal", "search"], "", "");

__defineStringReflections(HTMLMarqueeElement.prototype, [["bgColor", "bgcolor"], ["height"], ["width"]]);
__defineUnsignedReflections(HTMLMarqueeElement.prototype, [["hspace", "hspace", 0], ["scrollAmount", "scrollamount", 6], ["scrollDelay", "scrolldelay", 85], ["vspace", "vspace", 0]]);
__defineBooleanReflections(HTMLMarqueeElement.prototype, [["trueSpeed", "truespeed"]]);
__defineEnumReflection(HTMLMarqueeElement.prototype, "behavior", "behavior", ["scroll", "slide", "alternate"], "scroll", "scroll");
__defineEnumReflection(HTMLMarqueeElement.prototype, "direction", "direction", ["up", "right", "down", "left"], "left", "left");
__defineStringReflections(HTMLFrameSetElement.prototype, [["cols"], ["rows"]]);
__defineStringReflections(HTMLFrameElement.prototype, [["name"], ["scrolling"], ["frameBorder", "frameborder"], ["marginHeight", "marginheight"], ["marginWidth", "marginwidth"]]);
__defineUrlReflections(HTMLFrameElement.prototype, [["src"], ["longDesc", "longdesc"]]);
__defineBooleanReflections(HTMLFrameElement.prototype, [["noResize", "noresize"]]);
__defineNullAsEmptyStringReflection(HTMLFrameElement.prototype, "marginHeight", "marginheight");
__defineNullAsEmptyStringReflection(HTMLFrameElement.prototype, "marginWidth", "marginwidth");
__defineBooleanReflections(HTMLDirectoryElement.prototype, [["compact"]]);
__defineStringReflections(HTMLFontElement.prototype, [["color"], ["face"], ["size"]]);
__defineNullAsEmptyStringReflection(HTMLFontElement.prototype, "color", "color");

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
    querySelector(selector) { return __callHost("querySelector", this, selector); }
    querySelectorAll(selector) { return new NodeList(__callHost("querySelectorAll", this, selector)); }
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

function __urlRecord(input, base = undefined) {
    try {
        return JSON.parse(__callHost("urlParse", window, String(input), base === undefined ? "" : String(base)));
    } catch (_) {
        throw new TypeError("Invalid URL");
    }
}

class URLSearchParams {
    constructor(init = "", owner = null) {
        this.__owner = owner;
        if (typeof init === "string") {
            this.__pairs = JSON.parse(__callHost("urlSearchParamsParse", window, init));
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
    toString() { return __callHost("urlSearchParamsSerialize", window, JSON.stringify(this.__pairs)); }
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
        this.__href = __callHost("urlSet", window, this.__href, component, String(value));
        if (this.__searchParams) {
            this.__searchParams.__pairs = JSON.parse(__callHost("urlSearchParamsParse", window, this.search));
        }
    }
}
globalThis.URL = URL;
globalThis.URLSearchParams = URLSearchParams;

function __encodingBytes(input) {
    if (input === undefined) return [];
    if (input instanceof ArrayBuffer) return Array.from(new Uint8Array(input));
    if (typeof SharedArrayBuffer !== "undefined" && input instanceof SharedArrayBuffer) {
        return Array.from(new Uint8Array(input));
    }
    if (ArrayBuffer.isView(input)) {
        if (input.buffer instanceof ArrayBuffer && input.buffer.byteLength === 0) return [];
        return Array.from(new Uint8Array(input.buffer, input.byteOffset, input.byteLength));
    }
    throw new TypeError("TextDecoder input must be an ArrayBuffer or an ArrayBufferView");
}

function __iso2022JpStatePrefix(bytes) {
    let state = [];
    for (let index = 0; index < bytes.length; index++) {
        if (bytes[index] !== 0x1B) continue;
        if (bytes[index + 1] === 0x28 && [0x42, 0x49, 0x4A].includes(bytes[index + 2])) {
            state = bytes.slice(index, index + 3);
            index += 2;
        } else if (bytes[index + 1] === 0x24 && [0x40, 0x42].includes(bytes[index + 2])) {
            state = bytes.slice(index, index + 3);
            index += 2;
        } else if (bytes[index + 1] === 0x24 && bytes[index + 2] === 0x28 && bytes[index + 3] === 0x44) {
            state = bytes.slice(index, index + 4);
            index += 3;
        }
    }
    return state;
}

class TextDecoder {
    constructor(label = "utf-8", options = {}) {
        const encoding = __callHost("encodingCanonical", window, String(label));
        if (encoding === null) throw new RangeError("The encoding label is invalid");
        this.__encoding = encoding;
        this.__fatal = Boolean(options.fatal);
        this.__ignoreBOM = Boolean(options.ignoreBOM);
        this.__bytes = [];
        this.__emitted = 0;
        this.__streaming = false;
    }
    get encoding() { return this.__encoding; }
    get fatal() { return this.__fatal; }
    get ignoreBOM() { return this.__ignoreBOM; }
    decode(input = undefined, options = {}) {
        const stream = Boolean(options.stream);
        const bytes = __encodingBytes(input);
        if (this.__streaming) this.__bytes.push(...bytes);
        else this.__bytes = bytes;
        const decoded = __callHost(
            "decodeBytes",
            window,
            this.encoding,
            JSON.stringify(this.__bytes),
            this.fatal,
            this.ignoreBOM,
            stream,
        );
        if (decoded === null) {
            const preserveIso2022JpState = stream && this.encoding === "iso-2022-jp";
            this.__bytes = preserveIso2022JpState ? __iso2022JpStatePrefix(this.__bytes) : [];
            this.__emitted = 0;
            this.__streaming = preserveIso2022JpState;
            throw new TypeError("The encoded data is not valid");
        }
        const output = decoded.slice(this.__emitted);
        if (stream) {
            this.__emitted = decoded.length;
            this.__streaming = true;
        } else {
            this.__bytes = [];
            this.__emitted = 0;
            this.__streaming = false;
        }
        return output;
    }
}

class TextEncoder {
    get encoding() { return "utf-8"; }
    encode(input = "") {
        return new Uint8Array(JSON.parse(__callHost("encodeUtf8", window, __toUSVString(input))));
    }
    encodeInto(source, destination) {
        source = __toUSVString(source);
        if (!(destination instanceof Uint8Array)) {
            throw new TypeError("TextEncoder destination must be a Uint8Array");
        }
        let read = 0;
        let written = 0;
        for (const scalar of source) {
            const bytes = this.encode(scalar);
            if (written + bytes.length > destination.length) break;
            destination.set(bytes, written);
            written += bytes.length;
            read += scalar.length;
        }
        return { read, written };
    }
}

function __toUSVString(value) {
    value = String(value);
    let output = "";
    for (let index = 0; index < value.length; index++) {
        const first = value.charCodeAt(index);
        if (first >= 0xD800 && first <= 0xDBFF) {
            const second = value.charCodeAt(index + 1);
            if (second >= 0xDC00 && second <= 0xDFFF) {
                output += value[index] + value[index + 1];
                index++;
            } else {
                output += "\uFFFD";
            }
        } else if (first >= 0xDC00 && first <= 0xDFFF) {
            output += "\uFFFD";
        } else {
            output += value[index];
        }
    }
    return output;
}

globalThis.TextDecoder = TextDecoder;
globalThis.TextEncoder = TextEncoder;

globalThis.btoa = input => {
    input = String(input);
    const bytes = [];
    for (let index = 0; index < input.length; index++) {
        const value = input.charCodeAt(index);
        if (value > 255) {
            throw new DOMException("The string contains characters outside of Latin1", "InvalidCharacterError");
        }
        bytes.push(value);
    }
    return __callHost("base64Encode", window, JSON.stringify(bytes));
};

globalThis.atob = input => {
    const encoded = __callHost("base64Decode", window, String(input));
    if (encoded === null) {
        throw new DOMException("The string is not correctly encoded", "InvalidCharacterError");
    }
    const bytes = JSON.parse(encoded);
    let output = "";
    for (const byte of bytes) output += String.fromCharCode(byte);
    return output;
};

function __blobPartBytes(part, endings) {
    if (part instanceof Blob) return part.__bytes;
    if (part instanceof ArrayBuffer) return new Uint8Array(part);
    if (typeof SharedArrayBuffer !== "undefined" && part instanceof SharedArrayBuffer) {
        return new Uint8Array(part);
    }
    if (ArrayBuffer.isView(part)) {
        return new Uint8Array(part.buffer, part.byteOffset, part.byteLength);
    }
    let text = String(part);
    if (endings === "native") text = text.replace(/\r\n|\r/g, "\n");
    return new TextEncoder().encode(text);
}

function __isHttpToken(value) {
    if (value.length === 0) return false;
    for (let index = 0; index < value.length; index++) {
        const code = value.charCodeAt(index);
        if (code < 0x21 || code > 0x7E || "()<>@,;:\"/[]?={}\\".includes(value[index])) return false;
    }
    return true;
}

function __isHttpQuotedString(value) {
    for (let index = 0; index < value.length; index++) {
        const code = value.charCodeAt(index);
        if (code !== 0x09 && !(code >= 0x20 && code <= 0x7E) && !(code >= 0x80 && code <= 0xFF)) {
            return false;
        }
    }
    return true;
}

function __parseMimeType(input) {
    input = String(input).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
    const slash = input.indexOf("/");
    if (slash <= 0) return "";
    const type = input.slice(0, slash);
    if (!__isHttpToken(type)) return "";
    let position = slash + 1;
    let semicolon = input.indexOf(";", position);
    if (semicolon === -1) semicolon = input.length;
    const subtype = input.slice(position, semicolon).replace(/[\t\n\r ]+$/g, "");
    if (!__isHttpToken(subtype)) return "";
    const parameters = new Map();
    position = semicolon;
    while (position < input.length) {
        position++;
        while (/[\t\n\r ]/.test(input[position] ?? "")) position++;
        const nameStart = position;
        while (position < input.length && input[position] !== ";" && input[position] !== "=") position++;
        let name = input.slice(nameStart, position).toLowerCase();
        if (position >= input.length || input[position] === ";") continue;
        position++;
        let value = "";
        if (input[position] === '"') {
            position++;
            while (position < input.length) {
                const character = input[position++];
                if (character === '"') break;
                if (character === "\\" && position < input.length) value += input[position++];
                else value += character;
            }
            while (position < input.length && input[position] !== ";") position++;
        } else {
            const valueStart = position;
            while (position < input.length && input[position] !== ";") position++;
            value = input.slice(valueStart, position).replace(/[\t\n\r ]+$/g, "");
            if (value.length === 0) continue;
        }
        if (!parameters.has(name) && __isHttpToken(name) && __isHttpQuotedString(value)) {
            parameters.set(name, value);
        }
    }
    let output = type.toLowerCase() + "/" + subtype.toLowerCase();
    for (const [name, value] of parameters) {
        output += ";" + name + "=";
        output += __isHttpToken(value)
            ? value
            : '"' + value.replace(/(["\\])/g, "\\$1") + '"';
    }
    return output;
}

class Blob {
    constructor(blobParts = [], options = {}) {
        if (blobParts === null || (typeof blobParts !== "object" && typeof blobParts !== "function")) {
            throw new TypeError("blobParts must be a sequence");
        }
        const iterator = blobParts[Symbol.iterator];
        if (typeof iterator !== "function") throw new TypeError("blobParts must be a sequence");
        if (options == null) options = {};
        else if (typeof options !== "object" && typeof options !== "function") {
            throw new TypeError("options must be a dictionary");
        }
        const endings = options.endings === undefined ? "transparent" : String(options.endings);
        if (endings !== "transparent" && endings !== "native") {
            throw new TypeError("endings must be 'transparent' or 'native'");
        }
        const chunks = [];
        let size = 0;
        for (const part of blobParts) {
            const bytes = __blobPartBytes(part, endings);
            chunks.push(bytes);
            size += bytes.byteLength;
        }
        this.__bytes = new Uint8Array(size);
        let offset = 0;
        for (const chunk of chunks) {
            this.__bytes.set(chunk, offset);
            offset += chunk.byteLength;
        }
        this.__type = __parseMimeType(options.type === undefined ? "" : options.type);
    }
    get size() { return this.__bytes.byteLength; }
    get type() { return this.__type; }
    slice(start = 0, end = this.size, contentType = "") {
        const size = this.size;
        start = Number(start);
        end = Number(end);
        start = Number.isNaN(start) ? 0 : Math.trunc(start);
        end = Number.isNaN(end) ? 0 : Math.trunc(end);
        const first = start < 0 ? Math.max(size + start, 0) : Math.min(start, size);
        const last = end < 0 ? Math.max(size + end, 0) : Math.min(end, size);
        return new Blob([this.__bytes.slice(first, Math.max(first, last))], { type: contentType });
    }
    arrayBuffer() { return Promise.resolve(this.__bytes.slice().buffer); }
    bytes() { return Promise.resolve(this.__bytes.slice()); }
    text() { return Promise.resolve(new TextDecoder().decode(this.__bytes)); }
    get [Symbol.toStringTag]() { return "Blob"; }
}
globalThis.Blob = Blob;

class File extends Blob {
    constructor(fileBits, fileName, options = {}) {
        if (arguments.length < 2) throw new TypeError("File requires fileBits and fileName");
        if (options == null) options = {};
        super(fileBits, options);
        this.__name = __toUSVString(fileName);
        const lastModified = options.lastModified === undefined ? Date.now() : Number(options.lastModified);
        this.__lastModified = Number.isFinite(lastModified) ? Math.trunc(lastModified) : 0;
    }
    get name() { return this.__name; }
    get lastModified() { return this.__lastModified; }
    get webkitRelativePath() { return ""; }
    get [Symbol.toStringTag]() { return "File"; }
}
globalThis.File = File;

const __storageData = new WeakMap();
const __storageNamespaces = new WeakMap();
const __storageConstructorToken = {};

function __storageMap(receiver) {
    const map = __storageData.get(receiver);
    if (map === undefined) throw new TypeError("Storage method called on an incompatible receiver");
    return map;
}

function __storeStorageValue(receiver, key, value) {
    const map = __storageMap(receiver);
    const hadValue = map.has(key);
    const previous = map.get(key);
    map.set(key, value);
    const namespace = __storageNamespaces.get(receiver);
    if (namespace !== undefined) {
        try { __callHost("persistentSet", globalThis, namespace, key, value); }
        catch (error) {
            if (hadValue) map.set(key, previous);
            else map.delete(key);
            if (String(error).includes("QuotaExceededError")) {
                throw new DOMException("The quota has been exceeded", "QuotaExceededError");
            }
            throw error;
        }
    }
}

function __deleteStorageValue(receiver, key) {
    __storageMap(receiver).delete(key);
    const namespace = __storageNamespaces.get(receiver);
    if (namespace !== undefined) __callHost("persistentDelete", globalThis, namespace, key);
}

function __toUnsignedLong(value) {
    let number = Number(value);
    if (!Number.isFinite(number) || number === 0) return 0;
    number = Math.trunc(number) % 0x100000000;
    return number < 0 ? number + 0x100000000 : number;
}

const __storageProxyHandler = {
    get(target, property, receiver) {
        if (typeof property !== "string" || Reflect.has(target, property)) {
            return Reflect.get(target, property, receiver);
        }
        return __storageMap(target).get(property);
    },
    set(target, property, value, receiver) {
        if (typeof property !== "string") return Reflect.set(target, property, value, receiver);
        const converted = String(value);
        __storeStorageValue(target, property, converted);
        return true;
    },
    defineProperty(target, property, descriptor) {
        if (typeof property !== "string") return Reflect.defineProperty(target, property, descriptor);
        const converted = String(descriptor.value);
        __storeStorageValue(target, property, converted);
        return true;
    },
    deleteProperty(target, property) {
        if (typeof property !== "string") return Reflect.deleteProperty(target, property);
        __deleteStorageValue(target, property);
        return true;
    },
    has(target, property) {
        return Reflect.has(target, property) ||
            (typeof property === "string" && __storageMap(target).has(property));
    },
    ownKeys(target) {
        const named = [];
        for (const key of __storageMap(target).keys()) {
            if (!Reflect.has(target, key)) named.push(key);
        }
        return named.concat(Reflect.ownKeys(target).filter(key => typeof key === "symbol"));
    },
    getOwnPropertyDescriptor(target, property) {
        if (typeof property !== "string") return Reflect.getOwnPropertyDescriptor(target, property);
        const map = __storageMap(target);
        if (Reflect.has(target, property) || !map.has(property)) return undefined;
        return { value: map.get(property), writable: true, enumerable: true, configurable: true };
    },
};

class Storage {
    constructor(token, namespace = undefined) {
        if (token !== __storageConstructorToken) throw new TypeError("Illegal constructor");
        const map = new Map();
        const proxy = new Proxy(this, __storageProxyHandler);
        __storageData.set(this, map);
        __storageData.set(proxy, map);
        if (namespace !== undefined) {
            __storageNamespaces.set(this, namespace);
            __storageNamespaces.set(proxy, namespace);
            try {
                for (const key of JSON.parse(__callHost("persistentList", globalThis, namespace))) {
                    const value = __callHost("persistentGet", globalThis, namespace, key);
                    if (value !== null) map.set(key, value);
                }
            } catch (_) {}
        }
        return proxy;
    }
    get length() { return __storageMap(this).size; }
    key(index) {
        if (arguments.length === 0) throw new TypeError("Storage.key requires an index");
        return Array.from(__storageMap(this).keys())[__toUnsignedLong(index)] ?? null;
    }
    getItem(key) {
        if (arguments.length === 0) throw new TypeError("Storage.getItem requires a key");
        key = String(key);
        return __storageMap(this).has(key) ? __storageMap(this).get(key) : null;
    }
    setItem(key, value) {
        if (arguments.length < 2) throw new TypeError("Storage.setItem requires a key and value");
        key = String(key);
        value = String(value);
        __storeStorageValue(this, key, value);
    }
    removeItem(key) {
        if (arguments.length === 0) throw new TypeError("Storage.removeItem requires a key");
        __deleteStorageValue(this, String(key));
    }
    clear() {
        __storageMap(this).clear();
        const namespace = __storageNamespaces.get(this);
        if (namespace !== undefined) __callHost("persistentClear", globalThis, namespace);
    }
    get [Symbol.toStringTag]() { return "Storage"; }
}

globalThis.Storage = Storage;
globalThis.localStorage = new Storage(
    __storageConstructorToken,
    __runtimeFeatures.persistentStorage ? "localStorage" : undefined,
);
globalThis.sessionStorage = new Storage(__storageConstructorToken);

class Headers {
    constructor(init = undefined) {
        this.__values = new Map();
        if (init instanceof Headers) {
            for (const [name, value] of init) this.append(name, value);
        } else if (Array.isArray(init)) {
            for (const pair of init) {
                if (pair == null || pair.length !== 2) {
                    throw new TypeError("Each header pair must contain exactly two items");
                }
                this.append(pair[0], pair[1]);
            }
        } else if (init != null) {
            for (const name of Object.keys(init)) this.append(name, init[name]);
        }
    }
    append(name, value) {
        name = __headerName(name);
        value = __headerValue(value);
        const old = this.__values.get(name);
        this.__values.set(name, old === undefined ? value : `${old}, ${value}`);
    }
    set(name, value) { this.__values.set(__headerName(name), __headerValue(value)); }
    get(name) { return this.__values.get(__headerName(name)) ?? null; }
    has(name) { return this.__values.has(__headerName(name)); }
    delete(name) { this.__values.delete(__headerName(name)); }
    entries() { return this.__values.entries(); }
    keys() { return this.__values.keys(); }
    values() { return this.__values.values(); }
    forEach(callback, thisArg = undefined) {
        for (const [name, value] of this.__values) callback.call(thisArg, value, name, this);
    }
    [Symbol.iterator]() { return this.entries(); }
}

function __headerName(name) {
    name = String(name);
    if (!__isHttpToken(name)) throw new TypeError("Invalid HTTP header name");
    return name.toLowerCase();
}

function __headerValue(value) {
    value = String(value);
    for (let index = 0; index < value.length; index++) {
        if (value.charCodeAt(index) > 0xFF) {
            throw new TypeError("HTTP header values must be byte strings");
        }
    }
    value = value.replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
    if (/[\0\n\r]/.test(value)) throw new TypeError("Invalid HTTP header value");
    return value;
}

class Response {
    constructor(body = "", init = {}) {
        this.__body = body == null ? "" : String(body);
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
    arrayBuffer() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve(new TextEncoder().encode(this.__body).buffer);
    }
    blob() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve(new Blob([this.__body], { type: this.headers.get("content-type") || "" }));
    }
    clone() {
        if (this.bodyUsed) throw new TypeError("body has already been consumed");
        return new Response(this.__body, {
            status: this.status,
            statusText: this.statusText,
            headers: this.headers,
            url: this.url,
            redirected: this.redirected,
        });
    }
}

class Request {
    constructor(input, init = {}) {
        if (arguments.length === 0) throw new TypeError("Request input is required");
        const source = input instanceof Request ? input : null;
        if (source && source.bodyUsed) throw new TypeError("request body has already been consumed");
        const url = new URL(source ? source.url : String(input), location.href);
        if (url.username || url.password) throw new TypeError("request URL cannot contain credentials");
        this.url = url.href;
        let method = String(init.method ?? (source ? source.method : "GET"));
        if (!/^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/.test(method)) throw new TypeError("invalid request method");
        if (["CONNECT", "TRACE", "TRACK"].includes(method.toUpperCase())) {
            throw new TypeError("forbidden request method");
        }
        if (["DELETE", "GET", "HEAD", "OPTIONS", "POST", "PUT"].includes(method.toUpperCase())) {
            method = method.toUpperCase();
        }
        this.method = method;
        this.headers = new Headers(init.headers ?? (source ? source.headers : undefined));
        this.__body = init.body === undefined ? (source ? source.__body : null) : init.body;
        if ((this.method === "GET" || this.method === "HEAD") && this.__body != null) {
            throw new TypeError("GET and HEAD requests cannot have a body");
        }
        if (this.__body != null) this.__body = String(this.__body);
        this.bodyUsed = false;
        this.destination = source ? source.destination : "";
        this.referrer = String(init.referrer ?? (source ? source.referrer : "about:client"));
        this.referrerPolicy = String(init.referrerPolicy ?? (source ? source.referrerPolicy : ""));
        this.mode = String(init.mode ?? (source ? source.mode : "cors"));
        this.credentials = String(init.credentials ?? (source ? source.credentials : "same-origin"));
        this.cache = String(init.cache ?? (source ? source.cache : "default"));
        this.redirect = String(init.redirect ?? (source ? source.redirect : "follow"));
        this.integrity = String(init.integrity ?? (source ? source.integrity : ""));
        this.keepalive = Boolean(init.keepalive ?? (source ? source.keepalive : false));
        const sourceSignal = init.signal ?? (source ? source.signal : new AbortController().signal);
        if (!(sourceSignal instanceof AbortSignal)) throw new TypeError("signal must be an AbortSignal");
        this.signal = new AbortSignal();
        if (sourceSignal.aborted) this.signal.__abort(sourceSignal.reason);
        else sourceSignal.__dependents.push(this.signal);
        this.isReloadNavigation = false;
        this.isHistoryNavigation = false;
    }
    clone() {
        if (this.bodyUsed) throw new TypeError("request body has already been consumed");
        return new Request(this);
    }
    text() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve(this.__body == null ? "" : this.__body);
    }
    json() { return this.text().then(JSON.parse); }
    arrayBuffer() { return this.text().then(text => new TextEncoder().encode(text).buffer); }
    blob() {
        const type = this.headers.get("content-type") || "";
        return this.text().then(text => new Blob([text], { type }));
    }
}

globalThis.Headers = Headers;
globalThis.Response = Response;
globalThis.Request = Request;

globalThis.fetch = (input, init = {}) => {
    let request;
    try {
        request = new Request(input, init);
    } catch (error) {
        return Promise.reject(error);
    }
    return __callHost(
        "fetch",
        window,
        request.url,
        request.method,
        JSON.stringify([...request.headers]),
        request.__body,
    )
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

const __styleDeclarationTargets = new WeakMap();
const __styleDeclarationProxies = new WeakMap();
const __styleDeclarations = new WeakSet();

function __styleDeclarationTarget(declaration) {
    return __styleDeclarationTargets.get(declaration) ?? declaration;
}

function __requireStyleDeclaration(declaration) {
    const target = __styleDeclarationTarget(declaration);
    if ((typeof target !== "object" && typeof target !== "function") ||
        target === null || !__styleDeclarations.has(target)) {
        throw new TypeError("receiver is not a CSSStyleDeclaration");
    }
    return target;
}

function __requireWritableStyleDeclaration(declaration) {
    const target = __requireStyleDeclaration(declaration);
    if (!__callHost("styleWritable", target)) {
        throw new DOMException("The declaration is read-only", "NoModificationAllowedError");
    }
    return target;
}

function __styleDeclarationProxy(target) {
    let proxy = __styleDeclarationProxies.get(target);
    if (proxy !== undefined) return proxy;
    proxy = new Proxy(target, {
        get(target, property, receiver) {
            if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                const name = CSSStyleDeclaration.prototype.item.call(receiver, Number(property));
                return name === "" ? undefined : name;
            }
            if (typeof property === "string" && !(property in target)) {
                return CSSStyleDeclaration.prototype.getPropertyValue.call(
                    receiver,
                    __cssRulePropertyName(property),
                );
            }
            return Reflect.get(target, property, receiver);
        },
        set(target, property, value, receiver) {
            if (typeof property === "string" && !(property in target)) {
                CSSStyleDeclaration.prototype.setProperty.call(
                    receiver,
                    __cssRulePropertyName(property),
                    value,
                );
                return true;
            }
            return Reflect.set(target, property, value, receiver);
        },
        has(target, property) {
            if (Reflect.has(target, property)) return true;
            if (typeof property !== "string") return false;
            if (/^(0|[1-9][0-9]*)$/.test(property)) {
                return Number(property) < Reflect.get(target, "length", receiver);
            }
            return __cssSupportsDeclaration(__cssRulePropertyName(property), "initial");
        },
    });
    __styleDeclarations.add(target);
    __styleDeclarationTargets.set(proxy, target);
    __styleDeclarationProxies.set(target, proxy);
    return proxy;
}

class CSSStyleDeclaration {
    constructor() { throw new TypeError("Illegal constructor"); }
    __entries() {
        return JSON.parse(__callHost("styleDeclarations", __requireStyleDeclaration(this)));
    }
    get cssText() {
        return __callHost("styleCssText", __requireStyleDeclaration(this));
    }
    set cssText(value) {
        const target = __requireWritableStyleDeclaration(this);
        __callHost("styleSetCssText", target, String(value));
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
        return __callHost("styleGetProperty", __requireStyleDeclaration(this), name);
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
        const target = __requireWritableStyleDeclaration(this);
        name = String(name);
        value = value === null ? "" : String(value);
        priority = priority === null || priority === undefined ? "" : String(priority);
        if (priority && priority.toLowerCase() !== "important") return;
        if (!value) {
            this.removeProperty(name);
            return;
        }
        if (priority) {
            if (!__cssSupportsDeclaration(name, value)) return;
            this.removeProperty(name);
            this.cssText = `${this.cssText} ${name}: ${value} !important;`;
            return;
        }
        if (!__cssSupportsDeclaration(name, value)) return;
        this.removeProperty(name);
        __callHost("styleSetProperty", target, name, value);
    }
    removeProperty(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.removeProperty requires a property");
        }
        return __callHost("styleRemoveProperty", __requireWritableStyleDeclaration(this), name);
    }
    get parentRule() {
        __requireStyleDeclaration(this);
        return null;
    }
    [Symbol.iterator]() { return this.__entries().map(entry => entry[0])[Symbol.iterator](); }
}

class CSSStyleProperties extends CSSStyleDeclaration {
    get cssFloat() { return this.getPropertyValue("float"); }
    set cssFloat(value) { this.setProperty("float", value); }
}

Object.setPrototypeOf(CSSRuleStyleDeclaration.prototype, CSSStyleProperties.prototype);
Object.defineProperty(CSSRuleStyleDeclaration.prototype, "parentRule", {
    configurable: true,
    enumerable: true,
    get() { return this.__rule; },
});

let __cssSupportProbe = null;
function __cssSupportsDeclaration(property, value) {
    property = String(property).trim();
    value = String(value).trim();
    if (property === "" || value === "") return false;
    if (__cssSupportProbe === null) __cssSupportProbe = document.createElement("div");
    const style = __styleDeclarationTarget(__cssSupportProbe.style);
    __callHost("styleRemoveProperty", style, property);
    __callHost("styleSetProperty", style, property, value);
    const supported = __callHost("styleGetProperty", style, property) !== "";
    __callHost("styleRemoveProperty", style, property);
    return supported;
}

const CSS = {
    supports(propertyOrCondition, value = undefined) {
        if (arguments.length >= 2) return __cssSupportsDeclaration(propertyOrCondition, value);
        let condition = String(propertyOrCondition).trim();
        if (condition.startsWith("selector(") && condition.endsWith(")")) {
            try {
                document.querySelector(condition.slice(9, -1));
                return true;
            } catch (_) {
                return false;
            }
        }
        if (condition.startsWith("(") && condition.endsWith(")")) {
            condition = condition.slice(1, -1).trim();
        }
        const colon = condition.indexOf(":");
        return colon > 0 && __cssSupportsDeclaration(condition.slice(0, colon), condition.slice(colon + 1));
    },
    escape(value) {
        const input = String(value);
        const characters = Array.from(input);
        let output = "";
        for (let index = 0; index < characters.length; index++) {
            const character = characters[index];
            const code = character.codePointAt(0);
            if (code === 0) {
                output += "\uFFFD";
            } else if ((code >= 1 && code <= 31) || code === 127 ||
                       (index === 0 && code >= 48 && code <= 57) ||
                       (index === 1 && code >= 48 && code <= 57 && characters[0] === "-")) {
                output += `\\${code.toString(16)} `;
            } else if (index === 0 && character === "-" && characters.length === 1) {
                output += "\\-";
            } else if (code >= 128 || character === "-" || character === "_" ||
                       (code >= 48 && code <= 57) || (code >= 65 && code <= 90) ||
                       (code >= 97 && code <= 122)) {
                output += character;
            } else {
                output += `\\${character}`;
            }
        }
        return output;
    },
};
Object.defineProperty(CSS, Symbol.toStringTag, {
    value: "CSS",
    writable: false,
    enumerable: false,
    configurable: true,
});

class MutationRecord {
    constructor(type, target) {
        this.type = type;
        this.target = target;
        this.addedNodes = new NodeList([]);
        this.removedNodes = new NodeList([]);
        this.previousSibling = null;
        this.nextSibling = null;
        this.attributeName = null;
        this.attributeNamespace = null;
        this.oldValue = null;
    }
}

class MutationObserver {
    constructor(callback) {
        if (typeof callback !== "function") throw new TypeError("MutationObserver callback must be a function");
        this.__callback = callback;
        this.__target = null;
        this.__timer = null;
        this.__snapshot = "";
    }
    observe(target, options = {}) {
        if (!(target instanceof Node)) throw new TypeError("MutationObserver target must be a Node");
        if (!options.childList && !options.attributes && !options.characterData) {
            throw new TypeError("MutationObserver options must enable childList, attributes, or characterData");
        }
        this.disconnect();
        this.__target = target;
        this.__snapshot = this.__takeSnapshot();
        this.__schedule();
    }
    disconnect() {
        if (this.__timer !== null) clearTimeout(this.__timer);
        this.__timer = null;
        this.__target = null;
    }
    takeRecords() { return []; }
    __takeSnapshot() {
        if (!this.__target) return "";
        if (this.__target.nodeType === 9) {
            return this.__target.documentElement ? this.__target.documentElement.outerHTML : "";
        }
        return this.__target.outerHTML === undefined
            ? String(this.__target.textContent || "")
            : String(this.__target.outerHTML);
    }
    __schedule() {
        this.__timer = setTimeout(() => {
            if (!this.__target) return;
            const snapshot = this.__takeSnapshot();
            if (snapshot !== this.__snapshot) {
                this.__snapshot = snapshot;
                this.__callback([new MutationRecord("childList", this.__target)], this);
            }
            if (this.__target) this.__schedule();
        }, 16);
    }
}

class IntersectionObserverEntry {
    constructor(target, rootBounds, boundingClientRect, intersectionRect) {
        this.time = performance.now();
        this.target = target;
        this.rootBounds = rootBounds;
        this.boundingClientRect = boundingClientRect;
        this.intersectionRect = intersectionRect;
        const targetArea = boundingClientRect.width * boundingClientRect.height;
        const intersectionArea = intersectionRect.width * intersectionRect.height;
        this.intersectionRatio = targetArea === 0 ? 0 : intersectionArea / targetArea;
        this.isIntersecting = intersectionArea > 0;
    }
}

class IntersectionObserver {
    constructor(callback, options = {}) {
        if (typeof callback !== "function") throw new TypeError("IntersectionObserver callback must be a function");
        this.__callback = callback;
        this.root = options.root || null;
        this.rootMargin = String(options.rootMargin || "0px");
        const threshold = options.threshold === undefined ? [0] :
            (Array.isArray(options.threshold) ? options.threshold : [options.threshold]);
        this.thresholds = threshold.map(Number).sort((left, right) => left - right);
        if (this.thresholds.some(value => !Number.isFinite(value) || value < 0 || value > 1)) {
            throw new RangeError("IntersectionObserver threshold must be between 0 and 1");
        }
        this.__targets = new Set();
        this.__timer = null;
    }
    observe(target) {
        if (!(target instanceof Element)) throw new TypeError("IntersectionObserver target must be an Element");
        this.__targets.add(target);
        if (this.__timer === null) {
            this.__timer = setTimeout(() => {
                this.__timer = null;
                const rootBounds = this.root ? this.root.getBoundingClientRect() :
                    new DOMRect(0, 0, window.innerWidth, window.innerHeight);
                const entries = [...this.__targets].map(target => {
                    const rect = target.getBoundingClientRect();
                    const left = Math.max(rect.left, rootBounds.left);
                    const top = Math.max(rect.top, rootBounds.top);
                    const right = Math.min(rect.right, rootBounds.right);
                    const bottom = Math.min(rect.bottom, rootBounds.bottom);
                    const intersection = right > left && bottom > top
                        ? new DOMRect(left, top, right - left, bottom - top)
                        : new DOMRect();
                    return new IntersectionObserverEntry(target, rootBounds, rect, intersection);
                });
                if (entries.length) this.__callback(entries, this);
            }, 0);
        }
    }
    unobserve(target) { this.__targets.delete(target); }
    disconnect() {
        if (this.__timer !== null) clearTimeout(this.__timer);
        this.__timer = null;
        this.__targets.clear();
    }
    takeRecords() { return []; }
}

globalThis.DOMImplementation = DOMImplementation;
globalThis.CustomElementRegistry = CustomElementRegistry;
globalThis.customElements = customElements;
globalThis.DOMException = DOMException;
globalThis.MutationRecord = MutationRecord;
globalThis.MutationObserver = MutationObserver;
globalThis.IntersectionObserverEntry = IntersectionObserverEntry;
globalThis.IntersectionObserver = IntersectionObserver;
globalThis.EventTarget = EventTarget;
globalThis.Event = Event;
globalThis.CustomEvent = CustomEvent;
globalThis.UIEvent = UIEvent;
globalThis.MouseEvent = MouseEvent;
globalThis.KeyboardEvent = KeyboardEvent;
globalThis.MessageEvent = MessageEvent;
globalThis.StorageEvent = StorageEvent;
globalThis.MessagePort = MessagePort;
globalThis.MessageChannel = MessageChannel;
globalThis.postMessage = postMessage.bind(globalThis);
globalThis.AbortSignal = AbortSignal;
globalThis.AbortController = AbortController;
globalThis.DOMTokenList = DOMTokenList;
globalThis.HTMLCollection = HTMLCollection;
globalThis.NodeList = NodeList;
globalThis.NamedNodeMap = NamedNodeMap;
globalThis.Attr = Attr;
globalThis.Node = Node;
globalThis.Document = Document;
globalThis.HTMLDocument = Document;
globalThis.XMLDocument = XMLDocument;
globalThis.DOMParser = DOMParser;
globalThis.Element = Element;
globalThis.SVGElement = SVGElement;
globalThis.HTMLElement = HTMLElement;
globalThis.HTMLAnchorElement = HTMLAnchorElement;
globalThis.HTMLBaseElement = HTMLBaseElement;
globalThis.HTMLPictureElement = HTMLPictureElement;
globalThis.HTMLImageElement = HTMLImageElement;
globalThis.HTMLIFrameElement = HTMLIFrameElement;
globalThis.HTMLEmbedElement = HTMLEmbedElement;
globalThis.HTMLObjectElement = HTMLObjectElement;
globalThis.HTMLParamElement = HTMLParamElement;
globalThis.HTMLMediaElement = HTMLMediaElement;
globalThis.HTMLVideoElement = HTMLVideoElement;
globalThis.HTMLAudioElement = HTMLAudioElement;
globalThis.HTMLSourceElement = HTMLSourceElement;
globalThis.HTMLTrackElement = HTMLTrackElement;
globalThis.HTMLCanvasElement = HTMLCanvasElement;
globalThis.HTMLMapElement = HTMLMapElement;
globalThis.HTMLAreaElement = HTMLAreaElement;
globalThis.HTMLFormElement = HTMLFormElement;
globalThis.HTMLFieldSetElement = HTMLFieldSetElement;
globalThis.HTMLLegendElement = HTMLLegendElement;
globalThis.HTMLLabelElement = HTMLLabelElement;
globalThis.HTMLInputElement = HTMLInputElement;
globalThis.HTMLButtonElement = HTMLButtonElement;
globalThis.HTMLSelectElement = HTMLSelectElement;
globalThis.HTMLDataListElement = HTMLDataListElement;
globalThis.HTMLOptGroupElement = HTMLOptGroupElement;
globalThis.HTMLOptionElement = HTMLOptionElement;
globalThis.HTMLTextAreaElement = HTMLTextAreaElement;
globalThis.HTMLOutputElement = HTMLOutputElement;
globalThis.HTMLProgressElement = HTMLProgressElement;
globalThis.HTMLMeterElement = HTMLMeterElement;
globalThis.HTMLTableElement = HTMLTableElement;
globalThis.HTMLTableCaptionElement = HTMLTableCaptionElement;
globalThis.HTMLTableColElement = HTMLTableColElement;
globalThis.HTMLTableSectionElement = HTMLTableSectionElement;
globalThis.HTMLTableRowElement = HTMLTableRowElement;
globalThis.HTMLTableCellElement = HTMLTableCellElement;
globalThis.HTMLHeadElement = HTMLHeadElement;
globalThis.HTMLTitleElement = HTMLTitleElement;
globalThis.HTMLLinkElement = HTMLLinkElement;
globalThis.HTMLMetaElement = HTMLMetaElement;
globalThis.HTMLStyleElement = HTMLStyleElement;
globalThis.HTMLParagraphElement = HTMLParagraphElement;
globalThis.HTMLHRElement = HTMLHRElement;
globalThis.HTMLPreElement = HTMLPreElement;
globalThis.HTMLQuoteElement = HTMLQuoteElement;
globalThis.HTMLOListElement = HTMLOListElement;
globalThis.HTMLUListElement = HTMLUListElement;
globalThis.HTMLLIElement = HTMLLIElement;
globalThis.HTMLDListElement = HTMLDListElement;
globalThis.HTMLDivElement = HTMLDivElement;
globalThis.HTMLDataElement = HTMLDataElement;
globalThis.HTMLTimeElement = HTMLTimeElement;
globalThis.HTMLBRElement = HTMLBRElement;
globalThis.HTMLBodyElement = HTMLBodyElement;
globalThis.HTMLHeadingElement = HTMLHeadingElement;
globalThis.HTMLHtmlElement = HTMLHtmlElement;
globalThis.HTMLScriptElement = HTMLScriptElement;
globalThis.HTMLTemplateElement = HTMLTemplateElement;
globalThis.HTMLSlotElement = HTMLSlotElement;
globalThis.HTMLModElement = HTMLModElement;
globalThis.HTMLDetailsElement = HTMLDetailsElement;
globalThis.HTMLMenuElement = HTMLMenuElement;
globalThis.HTMLDialogElement = HTMLDialogElement;
globalThis.HTMLMarqueeElement = HTMLMarqueeElement;
globalThis.HTMLFrameSetElement = HTMLFrameSetElement;
globalThis.HTMLFrameElement = HTMLFrameElement;
globalThis.HTMLDirectoryElement = HTMLDirectoryElement;
globalThis.HTMLFontElement = HTMLFontElement;
globalThis.CharacterData = CharacterData;
globalThis.Text = Text;
globalThis.Comment = Comment;
globalThis.DocumentFragment = DocumentFragment;
globalThis.Range = Range;
globalThis.Selection = Selection;
globalThis.Window = Window;
globalThis.Location = Location;
globalThis.Navigator = Navigator;
globalThis.DOMRect = DOMRect;

function __makeWebIdlMembersEnumerable(prototype, names) {
    for (const name of names) {
        const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
        if (descriptor !== undefined && !descriptor.enumerable) {
            Object.defineProperty(prototype, name, { ...descriptor, enumerable: true });
        }
    }
}

function __tagWebIdlPrototype(constructor, name) {
    Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
        value: name,
        writable: false,
        enumerable: false,
        configurable: true,
    });
}

function __exposeWebIdl(name, value) {
    Object.defineProperty(globalThis, name, {
        value,
        writable: true,
        enumerable: false,
        configurable: true,
    });
}

for (const [constructor, name, members] of [
    [MediaList, "MediaList", ["mediaText", "length", "item", "appendMedium", "deleteMedium", "toString"]],
    [StyleSheet, "StyleSheet", ["type", "href", "ownerNode", "parentStyleSheet", "title", "media", "disabled"]],
    [CSSStyleSheet, "CSSStyleSheet", ["ownerRule", "cssRules", "insertRule", "deleteRule", "replace", "replaceSync", "rules", "addRule", "removeRule"]],
    [StyleSheetList, "StyleSheetList", ["item", "length"]],
    [CSSRuleList, "CSSRuleList", ["item", "length"]],
    [CSSRule, "CSSRule", ["cssText", "parentRule", "parentStyleSheet", "type"]],
    [CSSStyleRule, "CSSStyleRule", ["selectorText", "style"]],
    [CSSImportRule, "CSSImportRule", ["href", "media", "styleSheet", "layerName", "supportsText"]],
    [CSSGroupingRule, "CSSGroupingRule", ["cssRules", "insertRule", "deleteRule"]],
    [CSSConditionRule, "CSSConditionRule", ["conditionText"]],
    [CSSMediaRule, "CSSMediaRule", ["media"]],
    [CSSFontFaceRule, "CSSFontFaceRule", []],
    [CSSPageRule, "CSSPageRule", []],
    [CSSKeyframesRule, "CSSKeyframesRule", ["name", "cssRules", "appendRule", "deleteRule", "findRule", "length"]],
    [CSSKeyframeRule, "CSSKeyframeRule", ["keyText", "style"]],
    [CSSNamespaceRule, "CSSNamespaceRule", ["namespaceURI", "prefix"]],
    [CSSSupportsRule, "CSSSupportsRule", []],
    [CSSStyleDeclaration, "CSSStyleDeclaration", ["cssText", "length", "item", "getPropertyValue", "getPropertyPriority", "setProperty", "removeProperty", "parentRule"]],
    [CSSStyleProperties, "CSSStyleProperties", ["cssFloat"]],
]) {
    __makeWebIdlMembersEnumerable(constructor.prototype, members);
    __tagWebIdlPrototype(constructor, name);
    __exposeWebIdl(name, constructor);
}

__exposeWebIdl("CSS", CSS);

const __performanceTimeOrigin = Date.now();
class Performance extends EventTarget {
    now() { return Math.max(0, Date.now() - __performanceTimeOrigin); }
    get timeOrigin() { return __performanceTimeOrigin; }
    toJSON() { return { timeOrigin: this.timeOrigin }; }
}
globalThis.Performance = Performance;
globalThis.performance = new Performance();

globalThis.window = globalThis;
globalThis.self = globalThis;
globalThis.parent = globalThis;
globalThis.top = globalThis;
globalThis.opener = null;
globalThis.addEventListener = EventTarget.prototype.addEventListener.bind(globalThis);
globalThis.removeEventListener = EventTarget.prototype.removeEventListener.bind(globalThis);
globalThis.dispatchEvent = EventTarget.prototype.dispatchEvent.bind(globalThis);
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
globalThis.getComputedStyle = function getComputedStyle(element, pseudoElt = null) {
    if (arguments.length === 0) throw new TypeError("getComputedStyle requires an element");
    if (pseudoElt !== null) String(pseudoElt);
    return __styleDeclarationProxy(__callHost("getComputedStyle", element));
};
globalThis.getSelection = () => __selection;
globalThis.setTimeout = (callback, delay = 0) => __callHost("setTimeout", window, callback, delay);
globalThis.clearTimeout = id => __callHost("clearTimeout", window, id);
globalThis.queueMicrotask = callback => __callHost("queueMicrotask", window, callback);
let __nextAnimationFrameId = 1;
let __animationFrameScheduled = false;
const __animationFrameCallbacks = new Map();
function __runAnimationFrame() {
    __animationFrameScheduled = false;
    const callbacks = [...__animationFrameCallbacks];
    __animationFrameCallbacks.clear();
    const timestamp = performance.now();
    for (const [, callback] of callbacks) callback(timestamp);
}
globalThis.requestAnimationFrame = callback => {
    if (typeof callback !== "function") throw new TypeError("callback must be a function");
    const id = __nextAnimationFrameId++;
    __animationFrameCallbacks.set(id, callback);
    if (!__animationFrameScheduled) {
        __animationFrameScheduled = true;
        setTimeout(__runAnimationFrame, 16);
    }
    return id;
};
globalThis.cancelAnimationFrame = id => { __animationFrameCallbacks.delete(Number(id)); };
window.setTimeout = globalThis.setTimeout;
window.clearTimeout = globalThis.clearTimeout;
window.queueMicrotask = globalThis.queueMicrotask;
window.requestAnimationFrame = globalThis.requestAnimationFrame;
window.cancelAnimationFrame = globalThis.cancelAnimationFrame;

for (const name of Object.getOwnPropertyNames(globalThis)) {
    if (name === "__brimpMarkWebBuiltin") continue;
    const value = globalThis[name];
    if (typeof value === "function" && (!__initialGlobalNames.has(name) || /^[A-Z]/.test(name))) {
        __markWebBuiltinInterface(value);
        if (!__initialGlobalNames.has(name) && /^[A-Z]/.test(name)) {
            Object.defineProperty(globalThis, name, {
                value,
                writable: true,
                enumerable: false,
                configurable: true,
            });
        }
    }
}
for (const name of [
    "atob", "btoa", "fetch", "postMessage", "getComputedStyle", "getSelection",
    "setTimeout", "clearTimeout", "queueMicrotask", "requestAnimationFrame",
    "cancelAnimationFrame", "addEventListener", "removeEventListener", "dispatchEvent",
]) {
    __markWebBuiltin(globalThis[name], `function ${name}() { [native code] }`);
}
})();
