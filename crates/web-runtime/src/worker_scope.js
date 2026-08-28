const __listeners = new Map();
const __workerHost = globalThis.__brimpWorkerThread;
delete globalThis.__brimpWorkerThread;
const __nativeSources = new WeakMap();
const __originalFunctionToString = Function.prototype.toString;
Object.defineProperty(Function.prototype, "toString", {
    value: function toString() {
        return __nativeSources.get(this) ?? __originalFunctionToString.call(this);
    },
    writable: true,
    configurable: true,
});
const __markBuiltin = (value, name = value.name) => {
    if (typeof value === "function") __nativeSources.set(value, `function ${name || ""}() { [native code] }`);
};
__markBuiltin(Function.prototype.toString, "toString");
class WorkerGlobalScope {}
class DedicatedWorkerGlobalScope extends WorkerGlobalScope {}
class SharedWorkerGlobalScope extends WorkerGlobalScope {}
class ServiceWorkerGlobalScope extends WorkerGlobalScope {}
class WorkletGlobalScope {}
class PaintWorkletGlobalScope extends WorkletGlobalScope {}
for (const constructor of [WorkerGlobalScope, DedicatedWorkerGlobalScope, SharedWorkerGlobalScope, ServiceWorkerGlobalScope, WorkletGlobalScope, PaintWorkletGlobalScope]) {
    __markBuiltin(constructor);
    Object.defineProperty(constructor.prototype, Symbol.toStringTag, { value: constructor.name, configurable: true });
    globalThis[constructor.name] = constructor;
}
globalThis.self = globalThis;
globalThis.addEventListener = (type, callback) => {
    type = String(type);
    const callbacks = __listeners.get(type) || [];
    callbacks.push(callback);
    __listeners.set(type, callbacks);
};
globalThis.removeEventListener = (type, callback) => {
    const callbacks = __listeners.get(String(type)) || [];
    const index = callbacks.indexOf(callback);
    if (index >= 0) callbacks.splice(index, 1);
};
globalThis.postMessage = message => __workerHost("post", JSON.stringify(message));
globalThis.close = () => __workerHost("close");
globalThis.registerPaint = () => {};
globalThis.registerProcessor = () => {};
globalThis.Response = class Response {
    constructor(body = "", init = {}) {
        this.__body = String(body ?? "");
        this.status = Number(init.status ?? 200);
        this.statusText = String(init.statusText ?? "");
        this.headers = Object.entries(init.headers ?? {});
    }
    text() { return Promise.resolve(this.__body); }
    json() { return this.text().then(JSON.parse); }
};
globalThis.Request = class Request {
    constructor(record) { Object.assign(this, record); }
};
globalThis.onmessage = null;
globalThis.__brimpDispatchWorkerMessage = serialized => {
    const event = { type: "message", data: JSON.parse(serialized), target: globalThis, currentTarget: globalThis };
    const target = globalThis.__brimpSharedPort || globalThis;
    for (const callback of (__listeners.get("message") || []).slice()) callback.call(target, event);
    if (typeof target.onmessage === "function") target.onmessage.call(target, event);
};
globalThis.__brimpConnectShared = () => {
    const port = {
        onmessage: null,
        postMessage: globalThis.postMessage,
        start() {}, close() {},
        addEventListener(type, callback) { if (type === "message") globalThis.addEventListener(type, callback); },
        removeEventListener(type, callback) { globalThis.removeEventListener(type, callback); },
    };
    globalThis.__brimpSharedPort = port;
    const event = { type: "connect", ports: [port], target: globalThis };
    for (const callback of (__listeners.get("connect") || []).slice()) callback.call(globalThis, event);
    if (typeof globalThis.onconnect === "function") globalThis.onconnect.call(globalThis, event);
};
globalThis.__brimpDispatchLifecycle = type => {
    const event = { type, target: globalThis, waitUntil(promise) { Promise.resolve(promise).catch(() => {}); } };
    for (const callback of (__listeners.get(type) || []).slice()) callback.call(globalThis, event);
    const handler = globalThis[`on${type}`];
    if (typeof handler === "function") handler.call(globalThis, event);
};
globalThis.__brimpDispatchFetch = serialized => {
    const record = JSON.parse(serialized);
    let responsePromise = null;
    const event = {
        type: "fetch",
        request: new Request(record),
        target: globalThis,
        respondWith(value) { responsePromise = Promise.resolve(value); },
        waitUntil(promise) { Promise.resolve(promise).catch(() => {}); },
    };
    for (const callback of (__listeners.get("fetch") || []).slice()) callback.call(globalThis, event);
    if (typeof globalThis.onfetch === "function") globalThis.onfetch.call(globalThis, event);
    if (responsePromise !== null) responsePromise.then(async response => {
        __workerHost("fetchResponse", JSON.stringify({
            status: response.status,
            statusText: response.statusText,
            headers: response.headers,
            body: await response.text(),
        }));
    });
};
globalThis.__brimpConfigureWorkerScope = kind => {
    const constructor = kind === "shared" ? SharedWorkerGlobalScope
        : kind === "service" ? ServiceWorkerGlobalScope
        : kind === "worklet" ? PaintWorkletGlobalScope
        : DedicatedWorkerGlobalScope;
    Object.defineProperty(globalThis, "constructor", { value: constructor, configurable: true });
    Object.defineProperty(globalThis, Symbol.toStringTag, { value: constructor.name, configurable: true });
    if (kind === "worklet") {
        delete globalThis.onmessage;
        delete globalThis.close;
        delete globalThis.postMessage;
    }
};
for (const [name, value] of Object.entries({
    addEventListener: globalThis.addEventListener,
    removeEventListener: globalThis.removeEventListener,
    postMessage: globalThis.postMessage,
    close: globalThis.close,
    registerPaint: globalThis.registerPaint,
    registerProcessor: globalThis.registerProcessor,
})) __markBuiltin(value, name);
