const __navigatorServiceConstructionToken = Symbol("navigator service construction");
const __transientActivationDuration = 5000;
const __userActivationNow = Date.now.bind(Date);
let __lastUserActivation = -Infinity;
let __hasBeenUserActive = false;

class UserActivation {
    constructor(...args) {
        const token = args[0];
        if (token !== __navigatorServiceConstructionToken) throw new TypeError("Illegal constructor");
    }
    get hasBeenActive() { return __hasBeenUserActive; }
    get isActive() { return __userActivationNow() - __lastUserActivation < __transientActivationDuration; }
}

const __userActivation = new UserActivation(__navigatorServiceConstructionToken);

function __notifyUserActivation() {
    __hasBeenUserActive = true;
    __lastUserActivation = __userActivationNow();
}

const __lockData = new WeakMap();

class Lock {
    constructor(...args) {
        const [token, name, mode] = args;
        if (token !== __navigatorServiceConstructionToken) throw new TypeError("Illegal constructor");
        __lockData.set(this, { name, mode });
    }
    get name() { return __lockData.get(this).name; }
    get mode() { return __lockData.get(this).mode; }
}

const __heldLocks = [];
const __pendingLocks = [];

function __lockSnapshot(entry) {
    return { name: entry.name, mode: entry.mode, clientId: "0" };
}

function __removeLock(entries, entry) {
    const index = entries.indexOf(entry);
    if (index >= 0) entries.splice(index, 1);
}

function __lockCanRun(entry) {
    const held = __heldLocks.filter(lock => lock.name === entry.name);
    if (entry.mode === "exclusive") return held.length === 0;
    return held.every(lock => lock.mode === "shared");
}

function __finishLock(entry, settlement, value) {
    if (entry.state !== "held") return;
    entry.state = "settled";
    __removeLock(__heldLocks, entry);
    if (entry.signal && entry.abortHandler) {
        entry.signal.removeEventListener("abort", entry.abortHandler);
    }
    settlement(value);
    __scheduleLocks(entry.name);
}

function __grantLock(entry) {
    __removeLock(__pendingLocks, entry);
    entry.state = "held";
    __heldLocks.push(entry);
    const lock = new Lock(__navigatorServiceConstructionToken, entry.name, entry.mode);
    Promise.resolve()
        .then(() => entry.callback(lock))
        .then(
            value => __finishLock(entry, entry.resolve, value),
            error => __finishLock(entry, entry.reject, error),
        );
}

function __scheduleLocks(name) {
    while (true) {
        const entry = __pendingLocks.find(lock => lock.name === name);
        if (!entry || !__lockCanRun(entry)) return;
        __grantLock(entry);
        if (entry.mode === "exclusive") return;
    }
}

function __abortLock(entry, reason) {
    if (entry.state === "settled" || entry.state === "aborted") return;
    const wasHeld = entry.state === "held";
    entry.state = "aborted";
    __removeLock(__pendingLocks, entry);
    __removeLock(__heldLocks, entry);
    entry.reject(reason);
    if (wasHeld || !__heldLocks.some(lock => lock.name === entry.name)) {
        __scheduleLocks(entry.name);
    }
}

class LockManager {
    constructor(...args) {
        const token = args[0];
        if (token !== __navigatorServiceConstructionToken) throw new TypeError("Illegal constructor");
    }
    request(name, options, callback = undefined) {
        if (arguments.length < 2) throw new TypeError("LockManager.request requires a name and callback");
        name = String(name);
        if (name.startsWith("-")) {
            throw new DOMException("Lock names beginning with '-' are reserved", "NotSupportedError");
        }
        if (typeof options === "function") {
            callback = options;
            options = {};
        } else {
            options = options == null ? {} : Object(options);
        }
        if (typeof callback !== "function") throw new TypeError("The lock callback must be a function");
        const mode = options.mode === undefined ? "exclusive" : String(options.mode);
        if (mode !== "exclusive" && mode !== "shared") throw new TypeError("Invalid lock mode");
        const ifAvailable = Boolean(options.ifAvailable);
        const steal = Boolean(options.steal);
        if (steal && (ifAvailable || mode !== "exclusive")) {
            throw new DOMException("A steal request must be exclusive and cannot use ifAvailable", "NotSupportedError");
        }
        const signal = options.signal;
        if (signal !== undefined && !(signal instanceof AbortSignal)) {
            throw new TypeError("signal must be an AbortSignal");
        }
        if (signal?.aborted) return Promise.reject(signal.reason);

        if (steal) {
            const reason = new DOMException("The lock was stolen", "AbortError");
            for (const entry of [...__heldLocks, ...__pendingLocks]) {
                if (entry.name === name) __abortLock(entry, reason);
            }
        }

        const unavailable = !__lockCanRun({ name, mode })
            || __pendingLocks.some(entry => entry.name === name);
        if (ifAvailable && unavailable) return Promise.resolve().then(() => callback(null));

        return new Promise((resolve, reject) => {
            const entry = {
                name,
                mode,
                callback,
                signal,
                resolve,
                reject,
                state: "pending",
                abortHandler: null,
            };
            if (signal) {
                entry.abortHandler = () => __abortLock(entry, signal.reason);
                signal.addEventListener("abort", entry.abortHandler, { once: true });
            }
            __pendingLocks.push(entry);
            __scheduleLocks(name);
        });
    }
    query() {
        return Promise.resolve({
            held: __heldLocks.map(__lockSnapshot),
            pending: __pendingLocks.map(__lockSnapshot),
        });
    }
}

const __lockManager = new LockManager(__navigatorServiceConstructionToken);

Object.defineProperties(Navigator.prototype, {
    webdriver: { get() { return false; }, enumerable: true, configurable: true },
    userActivation: { get() { return __userActivation; }, enumerable: true, configurable: true },
    locks: { get() { return __lockManager; }, enumerable: true, configurable: true },
});

class Crypto {
    constructor() { throw new TypeError("Illegal constructor"); }
    getRandomValues(array) {
        if (!ArrayBuffer.isView(array) || array instanceof DataView
            || ![Int8Array, Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array,
                Int32Array, Uint32Array, BigInt64Array, BigUint64Array].some(type => array instanceof type)) {
            throw new DOMException("The provided ArrayBufferView is not an integer typed array", "TypeMismatchError");
        }
        if (array.byteLength > 65536) {
            throw new DOMException("The requested length exceeds 65,536 bytes", "QuotaExceededError");
        }
        const bytes = JSON.parse(__callHost("cryptoRandomBytes", this, array.byteLength));
        new Uint8Array(array.buffer, array.byteOffset, array.byteLength).set(bytes);
        return array;
    }
    randomUUID() {
        const bytes = this.getRandomValues(new Uint8Array(16));
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        const hex = Array.from(bytes, byte => byte.toString(16).padStart(2, "0"));
        return `${hex.slice(0, 4).join("")}-${hex.slice(4, 6).join("")}-${hex.slice(6, 8).join("")}-${hex.slice(8, 10).join("")}-${hex.slice(10).join("")}`;
    }
}

const __crypto = Object.create(Crypto.prototype);
Object.defineProperty(globalThis, "crypto", { value: __crypto, writable: false, configurable: true });
