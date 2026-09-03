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
        this.__bytes = body == null
            ? new Uint8Array()
            : body instanceof Uint8Array
                ? body.slice()
                : Array.isArray(body)
                    ? Uint8Array.from(body)
                    : new TextEncoder().encode(String(body));
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
        return Promise.resolve(new TextDecoder().decode(this.__bytes));
    }
    json() { return this.text().then(JSON.parse); }
    arrayBuffer() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve(this.__bytes.slice().buffer);
    }
    blob() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve(new Blob([this.__bytes], { type: this.headers.get("content-type") || "" }));
    }
    clone() {
        if (this.bodyUsed) throw new TypeError("body has already been consumed");
        return new Response(this.__bytes, {
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
        const body = init.body === undefined ? (source ? source.__bodyBytes : null) : init.body;
        if ((this.method === "GET" || this.method === "HEAD") && body != null) {
            throw new TypeError("GET and HEAD requests cannot have a body");
        }
        if (body == null) {
            this.__bodyBytes = null;
        } else if (body instanceof FormData) {
            const serialized = __serializeFormData(body);
            this.__bodyBytes = serialized.bytes;
            if (!this.headers.has("content-type")) this.headers.set("content-type", serialized.type);
        } else if (body instanceof Blob) {
            this.__bodyBytes = body.__bytes.slice();
            if (body.type && !this.headers.has("content-type")) this.headers.set("content-type", body.type);
        } else if (body instanceof URLSearchParams) {
            this.__bodyBytes = new TextEncoder().encode(body.toString());
            if (!this.headers.has("content-type")) {
                this.headers.set("content-type", "application/x-www-form-urlencoded;charset=UTF-8");
            }
        } else if (body instanceof ArrayBuffer || ArrayBuffer.isView(body)) {
            this.__bodyBytes = Uint8Array.from(__blobPartBytes(body, "transparent"));
        } else {
            this.__bodyBytes = new TextEncoder().encode(__toUSVString(body));
            if (!this.headers.has("content-type")) this.headers.set("content-type", "text/plain;charset=UTF-8");
        }
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
        return Promise.resolve(this.__bodyBytes == null ? "" : new TextDecoder().decode(this.__bodyBytes));
    }
    json() { return this.text().then(JSON.parse); }
    arrayBuffer() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        return Promise.resolve((this.__bodyBytes ?? new Uint8Array()).slice().buffer);
    }
    blob() {
        if (this.bodyUsed) return Promise.reject(new TypeError("body has already been consumed"));
        this.bodyUsed = true;
        const type = this.headers.get("content-type") || "";
        return Promise.resolve(new Blob([this.__bodyBytes ?? new Uint8Array()], { type }));
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
        request.__bodyBytes === null ? null : JSON.stringify(Array.from(request.__bodyBytes)),
    )
        .then(serialized => {
            const payload = JSON.parse(serialized);
            return new Response(payload.bytes ?? payload.body, payload);
        }, reason => { throw new TypeError(String(reason)); });
};

class XMLHttpRequestEventTarget extends EventTarget {
    constructor() {
        super();
        this.onloadstart = null;
        this.onprogress = null;
        this.onabort = null;
        this.onerror = null;
        this.onload = null;
        this.ontimeout = null;
        this.onloadend = null;
    }
}

class XMLHttpRequestUpload extends XMLHttpRequestEventTarget {}

function __xhrProgressEvent(type, loaded = 0, total = 0) {
    return new ProgressEvent(type, {
        lengthComputable: total > 0,
        loaded,
        total,
    });
}

class XMLHttpRequest extends XMLHttpRequestEventTarget {
    constructor() {
        super();
        this.onreadystatechange = null;
        this.readyState = XMLHttpRequest.UNSENT;
        this.response = null;
        this.responseText = "";
        this.responseXML = null;
        this.responseURL = "";
        this.status = 0;
        this.statusText = "";
        this.timeout = 0;
        this.upload = new XMLHttpRequestUpload();
        this.withCredentials = false;
        this.__responseType = "";
        this.__method = "GET";
        this.__url = "";
        this.__headers = new Headers();
        this.__sent = false;
        this.__overrideMimeType = null;
        this.__generation = 0;
        this.__timeoutId = null;
        this.__controller = null;
        this.__responseHeaders = new Headers();
    }

    get responseType() { return this.__responseType; }
    set responseType(value) {
        value = String(value);
        if (!["", "text", "arraybuffer", "blob", "document", "json"].includes(value)) {
            throw new TypeError(`Unsupported XMLHttpRequest responseType: ${value}`);
        }
        if (this.readyState === XMLHttpRequest.LOADING || this.readyState === XMLHttpRequest.DONE) {
            throw new DOMException("The object is in an invalid state", "InvalidStateError");
        }
        this.__responseType = value;
    }

    open(method, url, async = true, username = null, password = null) {
        if (arguments.length < 2) throw new TypeError("XMLHttpRequest.open requires a method and URL");
        method = String(method);
        if (!/^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/.test(method)) {
            throw new DOMException("Invalid HTTP method", "SyntaxError");
        }
        method = method.toUpperCase();
        if (["CONNECT", "TRACE", "TRACK"].includes(method)) {
            throw new DOMException("Forbidden HTTP method", "SecurityError");
        }
        if (!Boolean(async)) {
            throw new DOMException("Synchronous XMLHttpRequest is not supported", "NotSupportedError");
        }
        const parsed = new URL(String(url), location.href);
        if (username !== null) parsed.username = String(username);
        if (password !== null) parsed.password = String(password);
        this.__generation += 1;
        this.__clearTimeout();
        this.__method = method;
        this.__url = parsed.href;
        this.__headers = new Headers();
        this.__responseHeaders = new Headers();
        this.__sent = false;
        this.__resetResponse();
        this.readyState = XMLHttpRequest.OPENED;
        this.dispatchEvent(new Event("readystatechange"));
    }

    setRequestHeader(name, value) {
        if (this.readyState !== XMLHttpRequest.OPENED || this.__sent) {
            throw new DOMException("The object is in an invalid state", "InvalidStateError");
        }
        this.__headers.append(name, value);
    }

    getResponseHeader(name) {
        if (this.readyState < XMLHttpRequest.HEADERS_RECEIVED) return null;
        return this.__responseHeaders.get(name);
    }

    getAllResponseHeaders() {
        if (this.readyState < XMLHttpRequest.HEADERS_RECEIVED) return "";
        let serialized = "";
        for (const [name, value] of this.__responseHeaders) serialized += `${name}: ${value}\r\n`;
        return serialized;
    }

    overrideMimeType(mime) {
        if (this.readyState === XMLHttpRequest.LOADING || this.readyState === XMLHttpRequest.DONE) {
            throw new DOMException("The object is in an invalid state", "InvalidStateError");
        }
        this.__overrideMimeType = String(mime);
    }

    send(body = null) {
        if (this.readyState !== XMLHttpRequest.OPENED || this.__sent) {
            throw new DOMException("The object is in an invalid state", "InvalidStateError");
        }
        this.__sent = true;
        const generation = ++this.__generation;
        const controller = new AbortController();
        this.__controller = controller;
        this.dispatchEvent(__xhrProgressEvent("loadstart"));
        if (body !== null && this.__method !== "GET" && this.__method !== "HEAD") {
            this.upload.dispatchEvent(__xhrProgressEvent("loadstart"));
        } else {
            body = null;
        }
        if (this.timeout > 0) {
            this.__timeoutId = setTimeout(() => {
                if (generation !== this.__generation || !this.__sent) return;
                this.__terminate("timeout");
            }, Math.max(0, Number(this.timeout)));
        }
        fetch(this.__url, {
            method: this.__method,
            headers: this.__headers,
            body,
            credentials: this.withCredentials ? "include" : "same-origin",
            signal: controller.signal,
        }).then(response => {
            if (generation !== this.__generation || !this.__sent) return;
            this.status = response.status;
            this.statusText = response.statusText;
            this.responseURL = response.url;
            this.__responseHeaders = new Headers(response.headers);
            this.readyState = XMLHttpRequest.HEADERS_RECEIVED;
            this.dispatchEvent(new Event("readystatechange"));
            this.readyState = XMLHttpRequest.LOADING;
            this.dispatchEvent(new Event("readystatechange"));
            const bytes = response.__bytes.slice();
            this.__finishResponse(bytes);
        }, () => {
            if (generation !== this.__generation || !this.__sent) return;
            this.__terminate("error");
        });
    }

    abort() {
        const active = this.__sent || this.readyState === XMLHttpRequest.HEADERS_RECEIVED ||
            this.readyState === XMLHttpRequest.LOADING;
        this.__generation += 1;
        this.__clearTimeout();
        if (this.__controller) this.__controller.abort();
        this.__controller = null;
        this.__sent = false;
        this.__resetResponse();
        if (active) {
            this.readyState = XMLHttpRequest.DONE;
            this.dispatchEvent(new Event("readystatechange"));
            this.dispatchEvent(__xhrProgressEvent("abort"));
            this.dispatchEvent(__xhrProgressEvent("loadend"));
        }
        this.readyState = XMLHttpRequest.UNSENT;
    }

    __finishResponse(bytes) {
        const text = new TextDecoder().decode(bytes);
        const type = this.__responseType;
        this.responseText = type === "" || type === "text" ? text : "";
        this.responseXML = null;
        if (type === "arraybuffer") this.response = bytes.buffer;
        else if (type === "blob") {
            this.response = new Blob([bytes], { type: this.__responseMimeType() });
        } else if (type === "json") {
            try { this.response = text === "" ? null : JSON.parse(text); }
            catch (_) { this.response = null; }
        } else if (type === "document" || type === "") {
            const mime = this.__responseMimeType();
            if (/^(?:text\/html|application\/xhtml\+xml|(?:application|text)\/xml|image\/svg\+xml)(?:;|$)/i.test(mime)) {
                const parserType = /^text\/html(?:;|$)/i.test(mime) ? "text/html" : "application/xml";
                this.responseXML = new DOMParser().parseFromString(text, parserType);
            }
            this.response = type === "document" ? this.responseXML : text;
        } else this.response = text;
        this.__sent = false;
        this.__controller = null;
        this.__clearTimeout();
        this.readyState = XMLHttpRequest.DONE;
        this.dispatchEvent(new Event("readystatechange"));
        const total = bytes.byteLength;
        this.dispatchEvent(__xhrProgressEvent("progress", total, total));
        this.dispatchEvent(__xhrProgressEvent("load", total, total));
        this.dispatchEvent(__xhrProgressEvent("loadend", total, total));
    }

    __terminate(type) {
        this.__generation += 1;
        this.__clearTimeout();
        if (this.__controller) this.__controller.abort();
        this.__controller = null;
        this.__sent = false;
        this.__resetResponse();
        this.readyState = XMLHttpRequest.DONE;
        this.dispatchEvent(new Event("readystatechange"));
        this.dispatchEvent(__xhrProgressEvent(type));
        this.dispatchEvent(__xhrProgressEvent("loadend"));
    }

    __responseMimeType() {
        return this.__overrideMimeType ?? this.__responseHeaders.get("content-type") ?? "";
    }

    __resetResponse() {
        this.response = null;
        this.responseText = "";
        this.responseXML = null;
        this.responseURL = "";
        this.status = 0;
        this.statusText = "";
    }

    __clearTimeout() {
        if (this.__timeoutId !== null) clearTimeout(this.__timeoutId);
        this.__timeoutId = null;
    }
}

for (const [name, value] of Object.entries({
    UNSENT: 0,
    OPENED: 1,
    HEADERS_RECEIVED: 2,
    LOADING: 3,
    DONE: 4,
})) {
    Object.defineProperty(XMLHttpRequest, name, { value, enumerable: true });
    Object.defineProperty(XMLHttpRequest.prototype, name, { value, enumerable: true });
}

globalThis.XMLHttpRequestEventTarget = XMLHttpRequestEventTarget;
globalThis.XMLHttpRequestUpload = XMLHttpRequestUpload;
globalThis.XMLHttpRequest = XMLHttpRequest;
