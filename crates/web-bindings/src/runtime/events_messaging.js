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

