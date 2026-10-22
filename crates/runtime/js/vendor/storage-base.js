export default ({global}) => {
const __storageData = new WeakMap();
const __storageConstructorToken = {};

function __storageMap(receiver) {
    const map = __storageData.get(receiver);
    if (map === undefined) throw new TypeError("Storage method called on an incompatible receiver");
    return map;
}

function __storeStorageValue(receiver, key, value) { __storageMap(receiver).set(key, value); }
function __deleteStorageValue(receiver, key) { __storageMap(receiver).delete(key); }

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
    constructor(token) {
        if (token !== __storageConstructorToken) throw new TypeError("Illegal constructor");
        const map = new Map();
        const proxy = new Proxy(this, __storageProxyHandler);
        __storageData.set(this, map);
        __storageData.set(proxy, map);
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
    }
    get [Symbol.toStringTag]() { return "Storage"; }
}

globalThis.Storage = Storage;
globalThis.localStorage = new Storage(__storageConstructorToken);
globalThis.sessionStorage = new Storage(__storageConstructorToken);

const namespaces = new Map();
function call(operation, namespace, key, value) {
    if (operation === 'estimate') {
        const usage = [...namespaces.values()].reduce((sum, map) => sum + [...map].reduce((n, [k,v]) => n + 2 * (k.length + v.length), 0), 0);
        return JSON.stringify({usage, quota: null});
    }
    if (!namespaces.has(namespace)) namespaces.set(namespace, new Map());
    const map = namespaces.get(namespace);
    if (operation === 'get') return map.get(key) ?? null;
    if (operation === 'set') { map.set(key, value); return; }
    if (operation === 'delete') { map.delete(key); return; }
    if (operation === 'list') return JSON.stringify([...map.keys()]);
    throw new Error('unknown in-memory storage operation');
}
const load = (namespace, key, fallback) => {
    const value = call("get", namespace, String(key));
    return value === null ? fallback : JSON.parse(value);
};
const save = (namespace, key, value) => {
    try { call("set", namespace, String(key), JSON.stringify(value)); }
    catch (error) {
        if (String(error).includes("QuotaExceededError")) {
            throw new DOMException("The quota has been exceeded", "QuotaExceededError");
        }
        throw error;
    }
};

class Cache {
    constructor(name) { this.__name = name; }
    __load() { return load("cache", this.__name, {}); }
    match(request) {
        const key = new Request(request).url;
        const entry = this.__load()[key];
        return Promise.resolve(entry ? new Response(entry.body, entry) : undefined);
    }
    matchAll(request = undefined) {
        const entries = this.__load();
        const values = request === undefined ? Object.values(entries) : [entries[new Request(request).url]].filter(Boolean);
        return Promise.resolve(values.map(entry => new Response(entry.body, entry)));
    }
    put(request, response) {
        const key = new Request(request).url;
        if (!(response instanceof Response)) return Promise.reject(new TypeError("response must be a Response"));
        return response.clone().text().then(body => {
            const entries = this.__load();
            entries[key] = { body, status: response.status, statusText: response.statusText, headers: [...response.headers], url: response.url };
            save("cache", this.__name, entries);
        });
    }
    add(request) { return fetch(request).then(response => this.put(request, response)); }
    addAll(requests) { return Promise.all(Array.from(requests, request => this.add(request))).then(() => undefined); }
    delete(request) {
        const key = new Request(request).url;
        const entries = this.__load();
        const existed = Object.hasOwn(entries, key);
        delete entries[key];
        save("cache", this.__name, entries);
        return Promise.resolve(existed);
    }
    keys() { return Promise.resolve(Object.keys(this.__load()).map(url => new Request(url))); }
}

class CacheStorage {
    open(name) { return Promise.resolve(new Cache(String(name))); }
    has(name) { return Promise.resolve(call("get", "cache", String(name)) !== null); }
    delete(name) {
        name = String(name);
        const existed = call("get", "cache", name) !== null;
        call("delete", "cache", name);
        return Promise.resolve(existed);
    }
    keys() { return Promise.resolve(JSON.parse(call("list", "cache"))); }
    match(request) {
        return this.keys().then(async names => {
            for (const name of names) {
                const response = await new Cache(name).match(request);
                if (response) return response;
            }
            return undefined;
        });
    }
}

class StorageManager {
    estimate() { return Promise.resolve(JSON.parse(call("estimate"))); }
    persist() { return Promise.resolve(false); }
    persisted() { return Promise.resolve(false); }
}

globalThis.Cache = Cache;
globalThis.CacheStorage = CacheStorage;
globalThis.caches = new CacheStorage();
globalThis.StorageManager = StorageManager;
Object.defineProperty(Navigator.prototype, "storage", { value: new StorageManager(), enumerable: true, configurable: true });


return () => { localStorage.clear(); sessionStorage.clear(); namespaces.clear(); };
}
