(() => {
"use strict";

const host = globalThis.__brimpWorkerHost;
const call = (operation, ...arguments_) => host(operation, globalThis, ...arguments_);
const clone = value => JSON.parse(JSON.stringify(value));
const workers = new Map();

class Worker extends EventTarget {
    constructor(scriptURL, options = {}) {
        super();
        if (arguments.length === 0) throw new TypeError("Worker script URL is required");
        this.onmessage = null;
        this.onerror = null;
        this.onmessageerror = null;
        this.__url = new URL(String(scriptURL), location.href).href;
        this.__name = String(options.name ?? "");
        this.__type = String(options.type ?? "classic");
        this.__kind = String(options.__kind ?? "dedicated");
        this.__scope = String(options.__scope ?? "");
        this.__id = call("workerCreate", this.__url, this.__kind, this.__name, this.__scope);
        workers.set(this.__id, serialized => {
            const envelope = JSON.parse(serialized);
            if (envelope.type === "message") {
                const event = new MessageEvent("message", { data: envelope.data });
                event.isTrusted = true;
                this.dispatchEvent(event);
            } else if (envelope.type === "ready") {
                this.dispatchEvent(new Event("load"));
            } else {
                const event = new ErrorEvent("error", { message: envelope.message });
                event.isTrusted = true;
                this.dispatchEvent(event);
            }
        });
    }
    postMessage(message) {
        if (arguments.length === 0) throw new TypeError("Worker.postMessage requires a message");
        call("workerPost", this.__id, JSON.stringify(clone(message)));
    }
    terminate() { workers.delete(this.__id); call("workerTerminate", this.__id); }
}

class ErrorEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        this.message = String(options.message ?? "");
        this.filename = String(options.filename ?? "");
        this.lineno = Number(options.lineno ?? 0);
        this.colno = Number(options.colno ?? 0);
        this.error = options.error ?? null;
    }
}

class SharedWorker extends EventTarget {
    constructor(scriptURL, options = {}) {
        super();
        const worker = new Worker(scriptURL, { ...options, __kind: "shared" });
        this.port = {
            onmessage: null,
            onmessageerror: null,
            start() {},
            close() { worker.terminate(); },
            postMessage(message) { worker.postMessage(message); },
            addEventListener(...arguments_) { worker.addEventListener(...arguments_); },
            removeEventListener(...arguments_) { worker.removeEventListener(...arguments_); },
        };
        worker.addEventListener("message", event => {
            if (typeof this.port.onmessage === "function") this.port.onmessage(event);
        });
    }
}

class ServiceWorker extends EventTarget {
    constructor(worker, scriptURL, state = "activated") {
        super();
        this.__worker = worker;
        this.scriptURL = scriptURL;
        this.state = state;
        this.onstatechange = null;
    }
    postMessage(message) { this.__worker.postMessage(message); }
}

class ServiceWorkerRegistration extends EventTarget {
    constructor(worker, scope) {
        super();
        this.scope = scope;
        this.installing = null;
        this.waiting = null;
        this.active = worker;
        this.onupdatefound = null;
    }
    update() { return Promise.resolve(this); }
    unregister() {
        if (!this.active) return Promise.resolve(false);
        workers.delete(this.active.__worker.__id);
        call("workerUnregister", this.active.__worker.__id);
        this.active = null;
        return Promise.resolve(true);
    }
}

class ServiceWorkerContainer extends EventTarget {
    constructor() {
        super();
        this.controller = null;
        this.oncontrollerchange = null;
        this.onmessage = null;
        this.onmessageerror = null;
        this.__registrations = new Map();
        this.ready = new Promise(resolve => { this.__resolveReady = resolve; });
    }
    register(scriptURL, options = {}) {
        const url = new URL(String(scriptURL), location.href).href;
        const scope = new URL(String(options.scope ?? "./"), url).href;
        const dedicated = new Worker(url, { type: options.type ?? "classic", __kind: "service", __scope: scope });
        const worker = new ServiceWorker(dedicated, url);
        const registration = new ServiceWorkerRegistration(worker, scope);
        this.__registrations.set(scope, registration);
        dedicated.addEventListener("message", event => this.dispatchEvent(new MessageEvent("message", { data: event.data, source: worker })));
        return new Promise((resolve, reject) => {
            dedicated.addEventListener("load", () => {
                this.controller = worker;
                this.__resolveReady(registration);
                resolve(registration);
            }, { once: true });
            dedicated.addEventListener("error", event => reject(event.error ?? new Error(event.message)), { once: true });
        });
    }
    getRegistration(clientURL = location.href) {
        clientURL = String(clientURL);
        return Promise.resolve([...this.__registrations.values()].find(registration => clientURL.startsWith(registration.scope)));
    }
    getRegistrations() { return Promise.resolve([...this.__registrations.values()]); }
    startMessages() {}
}

class Worklet {
    constructor() { this.__workers = []; }
    addModule(moduleURL) {
        const worker = new Worker(moduleURL, { type: "module", __kind: "worklet" });
        this.__workers.push(worker);
        return new Promise((resolve, reject) => {
            worker.addEventListener("load", resolve, { once: true });
            worker.addEventListener("error", event => reject(event.error ?? new Error(event.message)), { once: true });
        });
    }
}

globalThis.ErrorEvent = ErrorEvent;
globalThis.Worker = Worker;
globalThis.SharedWorker = SharedWorker;
globalThis.ServiceWorker = ServiceWorker;
globalThis.ServiceWorkerRegistration = ServiceWorkerRegistration;
globalThis.ServiceWorkerContainer = ServiceWorkerContainer;
globalThis.Worklet = Worklet;
Object.defineProperty(globalThis, "__brimpDeliverWorker", {
    value(serialized) {
        const delivery = JSON.parse(serialized);
        workers.get(Number(delivery.id))?.(delivery.event);
    },
    configurable: true,
});
Object.defineProperty(Navigator.prototype, "serviceWorker", {
    value: new ServiceWorkerContainer(),
    enumerable: true,
    configurable: true,
});
if (globalThis.CSS && typeof globalThis.CSS === "object") {
    Object.defineProperty(globalThis.CSS, "paintWorklet", { value: new Worklet(), configurable: true });
}
for (const constructor of [ErrorEvent, Worker, SharedWorker, ServiceWorker, ServiceWorkerRegistration, ServiceWorkerContainer, Worklet]) {
    globalThis.__brimpMarkWebBuiltin?.(constructor);
    for (const key of Reflect.ownKeys(constructor.prototype)) {
        const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
        if (typeof descriptor?.value === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.value);
        if (typeof descriptor?.get === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.get, `function get ${String(key)}() { [native code] }`);
    }
}
})();
