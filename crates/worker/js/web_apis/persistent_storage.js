(() => {
"use strict";

const host = globalThis.__brimpStorageHost;
const call = (operation, ...arguments_) => host(operation, globalThis, ...arguments_);
const clone = value => JSON.parse(JSON.stringify(value));
const load = (namespace, key, fallback) => {
    const value = call("persistentGet", namespace, String(key));
    return value === null ? fallback : JSON.parse(value);
};
const save = (namespace, key, value) => {
    try { call("persistentSet", namespace, String(key), JSON.stringify(value)); }
    catch (error) {
        if (String(error).includes("QuotaExceededError")) {
            throw new DOMException("The quota has been exceeded", "QuotaExceededError");
        }
        throw error;
    }
};
const later = callback => setTimeout(callback, 0);

class IDBRequest extends EventTarget {
    constructor() {
        super();
        this.result = undefined;
        this.error = null;
        this.source = null;
        this.transaction = null;
        this.readyState = "pending";
        this.onsuccess = null;
        this.onerror = null;
    }
    __success(result) {
        this.result = result;
        this.readyState = "done";
        this.dispatchEvent(new Event("success"));
    }
    __failure(error) {
        this.error = error;
        this.readyState = "done";
        this.dispatchEvent(new Event("error", { cancelable: true }));
    }
}

class IDBOpenDBRequest extends IDBRequest {
    constructor() {
        super();
        this.onupgradeneeded = null;
        this.onblocked = null;
        this.transaction = null;
    }
}

class DOMStringList {
    constructor(values) { this.__values = values; }
    get length() { return this.__values.length; }
    item(index) { return this.__values[Number(index)] ?? null; }
    contains(value) { return this.__values.includes(String(value)); }
    [Symbol.iterator]() { return this.__values[Symbol.iterator](); }
}

const keyText = key => JSON.stringify(key);
const extractKey = (value, keyPath) => keyPath == null ? undefined : value?.[keyPath];

class IDBObjectStore {
    constructor(transaction, name) {
        this.transaction = transaction;
        this.name = name;
    }
    get __record() { return this.transaction.db.__data.stores[this.name]; }
    get keyPath() { return this.__record.keyPath; }
    get autoIncrement() { return this.__record.autoIncrement; }
    get indexNames() { return new DOMStringList(Object.keys(this.__record.indexes ?? {}).sort()); }
    __request(operation) {
        const request = new IDBRequest();
        request.source = this;
        request.transaction = this.transaction;
        this.transaction.__beginRequest();
        later(() => {
            try { request.__success(operation()); }
            catch (error) { request.__failure(error); }
            finally { this.transaction.__endRequest(); }
        });
        return request;
    }
    __write(value, key, overwrite) {
        if (this.transaction.mode === "readonly") throw new DOMException("Transaction is read-only", "ReadOnlyError");
        return this.__request(() => {
            const record = this.__record;
            let resolved = key === undefined ? extractKey(value, record.keyPath) : key;
            if (resolved === undefined && record.autoIncrement) resolved = record.nextKey++;
            if (resolved === undefined) throw new DOMException("A key is required", "DataError");
            const encoded = keyText(resolved);
            if (!overwrite && Object.hasOwn(record.records, encoded)) {
                throw new DOMException("The key already exists", "ConstraintError");
            }
            for (const index of Object.values(record.indexes ?? {})) {
                if (!index.unique) continue;
                const indexKey = keyText(value?.[index.keyPath]);
                for (const [primaryKey, existing] of Object.entries(record.records)) {
                    if (primaryKey !== encoded && keyText(existing?.[index.keyPath]) === indexKey) {
                        throw new DOMException("The index key already exists", "ConstraintError");
                    }
                }
            }
            record.records[encoded] = clone(value);
            this.transaction.db.__save();
            return resolved;
        });
    }
    add(value, key = undefined) { return this.__write(value, key, false); }
    put(value, key = undefined) { return this.__write(value, key, true); }
    get(key) { return this.__request(() => clone(this.__record.records[keyText(key)])); }
    getKey(key) { return this.__request(() => Object.hasOwn(this.__record.records, keyText(key)) ? key : undefined); }
    getAll() { return this.__request(() => Object.values(this.__record.records).map(clone)); }
    getAllKeys() { return this.__request(() => Object.keys(this.__record.records).map(JSON.parse)); }
    count() { return this.__request(() => Object.keys(this.__record.records).length); }
    openCursor(query = undefined) { return this.__cursorRequest(query, false); }
    openKeyCursor(query = undefined) { return this.__cursorRequest(query, true); }
    __cursorRequest(query, keysOnly) {
        const entries = Object.entries(this.__record.records)
            .map(([key, value]) => [JSON.parse(key), clone(value)])
            .filter(([key]) => query === undefined || keyText(key) === keyText(query));
        return IDBCursor.__request(this, this.transaction, entries, keysOnly);
    }
    createIndex(name, keyPath, options = {}) {
        if (this.transaction.mode !== "versionchange") throw new DOMException("Not a versionchange transaction", "InvalidStateError");
        name = String(name);
        this.__record.indexes ??= {};
        if (this.__record.indexes[name]) throw new DOMException("Index exists", "ConstraintError");
        const index = { keyPath: String(keyPath), unique: Boolean(options.unique), multiEntry: Boolean(options.multiEntry) };
        if (index.unique) {
            const observed = new Set();
            for (const value of Object.values(this.__record.records)) {
                const key = keyText(value?.[index.keyPath]);
                if (observed.has(key)) throw new DOMException("Duplicate index key", "ConstraintError");
                observed.add(key);
            }
        }
        this.__record.indexes[name] = index;
        this.transaction.db.__save();
        return new IDBIndex(this, name);
    }
    index(name) {
        name = String(name);
        if (!this.__record.indexes?.[name]) throw new DOMException("Index not found", "NotFoundError");
        return new IDBIndex(this, name);
    }
    deleteIndex(name) {
        name = String(name);
        if (!this.__record.indexes?.[name]) throw new DOMException("Index not found", "NotFoundError");
        delete this.__record.indexes[name];
        this.transaction.db.__save();
    }
    delete(key) {
        return this.__request(() => {
            delete this.__record.records[keyText(key)];
            this.transaction.db.__save();
        });
    }
    clear() {
        return this.__request(() => {
            this.__record.records = {};
            this.transaction.db.__save();
        });
    }
}

class IDBIndex {
    constructor(objectStore, name) { this.objectStore = objectStore; this.name = name; }
    get __record() { return this.objectStore.__record.indexes[this.name]; }
    get keyPath() { return this.__record.keyPath; }
    get multiEntry() { return this.__record.multiEntry; }
    get unique() { return this.__record.unique; }
    __entries(query = undefined) {
        return Object.entries(this.objectStore.__record.records)
            .map(([primaryKey, value]) => [value?.[this.keyPath], JSON.parse(primaryKey), clone(value)])
            .filter(([key]) => query === undefined || keyText(key) === keyText(query));
    }
    get(query) { return this.objectStore.__request(() => this.__entries(query)[0]?.[2]); }
    getKey(query) { return this.objectStore.__request(() => this.__entries(query)[0]?.[1]); }
    getAll(query = undefined) { return this.objectStore.__request(() => this.__entries(query).map(entry => entry[2])); }
    getAllKeys(query = undefined) { return this.objectStore.__request(() => this.__entries(query).map(entry => entry[1])); }
    count(query = undefined) { return this.objectStore.__request(() => this.__entries(query).length); }
    openCursor(query = undefined) {
        return IDBCursor.__request(this, this.objectStore.transaction, this.__entries(query).map(([key, primary, value]) => [key, primary, value]), false, true);
    }
    openKeyCursor(query = undefined) {
        return IDBCursor.__request(this, this.objectStore.transaction, this.__entries(query).map(([key, primary]) => [key, primary, undefined]), true, true);
    }
}

class IDBCursor {
    static __request(source, transaction, entries, keysOnly, indexed = false) {
        const request = new IDBRequest();
        request.source = source;
        request.transaction = transaction;
        transaction.__beginRequest();
        let position = 0;
        const advance = (count = 1) => {
            position += count - 1;
            later(() => {
            if (position >= entries.length) {
                request.__success(null);
                transaction.__endRequest();
                return;
            }
            const entry = entries[position++];
            request.__success(new IDBCursor(request, entry, keysOnly, indexed, advance));
            });
        };
        advance();
        return request;
    }
    constructor(request, entry, keysOnly, indexed, advance) {
        this.request = request;
        this.key = entry[0];
        this.primaryKey = indexed ? entry[1] : entry[0];
        if (!keysOnly) this.value = indexed ? entry[2] : entry[1];
        this.__advance = advance;
    }
    continue() { this.__advance(1); }
    advance(count) {
        count = Number(count);
        if (!Number.isInteger(count) || count <= 0) throw new TypeError("count must be positive");
        this.__advance(count);
    }
}

class IDBTransaction extends EventTarget {
    constructor(db, names, mode) {
        super();
        this.db = db;
        this.mode = mode;
        this.objectStoreNames = new DOMStringList(names);
        this.error = null;
        this.oncomplete = null;
        this.onabort = null;
        this.onerror = null;
        this.__pending = 0;
        this.__finished = false;
        later(() => this.__maybeComplete());
    }
    objectStore(name) {
        name = String(name);
        if (!this.objectStoreNames.contains(name) || !this.db.__data.stores[name]) {
            throw new DOMException("Object store not found", "NotFoundError");
        }
        return new IDBObjectStore(this, name);
    }
    abort() { this.dispatchEvent(new Event("abort")); }
    commit() {}
    __beginRequest() {
        if (this.__finished) throw new DOMException("Transaction is inactive", "TransactionInactiveError");
        this.__pending++;
    }
    __endRequest() { this.__pending--; this.__maybeComplete(); }
    __maybeComplete() {
        if (!this.__finished && this.__pending === 0) {
            this.__finished = true;
            this.dispatchEvent(new Event("complete"));
        }
    }
}

class IDBDatabase extends EventTarget {
    constructor(name, data) {
        super();
        this.name = name;
        this.__data = data;
        this.onabort = null;
        this.onclose = null;
        this.onerror = null;
        this.onversionchange = null;
    }
    get version() { return this.__data.version; }
    get objectStoreNames() { return new DOMStringList(Object.keys(this.__data.stores).sort()); }
    __save() { save("indexeddb", this.name, this.__data); }
    createObjectStore(name, options = {}) {
        name = String(name);
        if (this.__data.stores[name]) throw new DOMException("Object store exists", "ConstraintError");
        this.__data.stores[name] = {
            keyPath: options.keyPath == null ? null : String(options.keyPath),
            autoIncrement: Boolean(options.autoIncrement),
            nextKey: 1,
            records: {},
            indexes: {},
        };
        this.__save();
        return new IDBObjectStore(new IDBTransaction(this, [name], "versionchange"), name);
    }
    deleteObjectStore(name) {
        name = String(name);
        if (!this.__data.stores[name]) throw new DOMException("Object store not found", "NotFoundError");
        delete this.__data.stores[name];
        this.__save();
    }
    transaction(storeNames, mode = "readonly") {
        const names = typeof storeNames === "string" ? [storeNames] : Array.from(storeNames, String);
        for (const name of names) if (!this.__data.stores[name]) throw new DOMException("Object store not found", "NotFoundError");
        return new IDBTransaction(this, names, String(mode));
    }
    close() {}
}

class IDBFactory {
    open(name, version = undefined) {
        name = String(name);
        const request = new IDBOpenDBRequest();
        later(() => {
            try {
                const old = load("indexeddb", name, null);
                const oldVersion = old?.version ?? 0;
                const nextVersion = version === undefined ? Math.max(1, oldVersion) : Number(version);
                if (!Number.isInteger(nextVersion) || nextVersion <= 0) throw new TypeError("version must be positive");
                if (nextVersion < oldVersion) throw new DOMException("Version is too low", "VersionError");
                const db = new IDBDatabase(name, old ?? { version: nextVersion, stores: {} });
                request.result = db;
                if (nextVersion > oldVersion) {
                    db.__data.version = nextVersion;
                    request.transaction = new IDBTransaction(db, Object.keys(db.__data.stores), "versionchange");
                    const event = new Event("upgradeneeded");
                    event.oldVersion = oldVersion;
                    event.newVersion = nextVersion;
                    request.dispatchEvent(event);
                    db.__save();
                }
                request.__success(db);
            } catch (error) { request.__failure(error); }
        });
        return request;
    }
    deleteDatabase(name) {
        const request = new IDBOpenDBRequest();
        later(() => {
            try { call("persistentDelete", "indexeddb", String(name)); request.__success(undefined); }
            catch (error) { request.__failure(error); }
        });
        return request;
    }
    databases() {
        return Promise.resolve(JSON.parse(call("persistentList", "indexeddb")).map(name => {
            const data = load("indexeddb", name, { version: 1 });
            return { name, version: data.version };
        }));
    }
    cmp(first, second) {
        const a = keyText(first), b = keyText(second);
        return a < b ? -1 : a > b ? 1 : 0;
    }
}

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
    has(name) { return Promise.resolve(call("persistentGet", "cache", String(name)) !== null); }
    delete(name) {
        name = String(name);
        const existed = call("persistentGet", "cache", name) !== null;
        call("persistentDelete", "cache", name);
        return Promise.resolve(existed);
    }
    keys() { return Promise.resolve(JSON.parse(call("persistentList", "cache"))); }
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

class FileSystemFileHandle {
    constructor(path) { this.kind = "file"; this.name = path.split("/").pop(); this.__path = path; }
    isSameEntry(other) { return Promise.resolve(other instanceof FileSystemFileHandle && other.__path === this.__path); }
    getFile() {
        const record = load("opfs", this.__path, { bytes: [], lastModified: Date.now() });
        return Promise.resolve(new File([new Uint8Array(record.bytes)], this.name, { lastModified: record.lastModified }));
    }
    createWritable() {
        if (__opfsWritableLocks.has(this.__path)) return Promise.reject(new DOMException("File is already open", "NoModificationAllowedError"));
        __opfsWritableLocks.add(this.__path);
        return Promise.resolve(new FileSystemWritableFileStream(this.__path));
    }
}

const __opfsWritableLocks = new Set();
class FileSystemWritableFileStream {
    constructor(path) {
        this.__path = path;
        this.__bytes = load("opfs", path, { bytes: [] }).bytes;
        this.__position = 0;
    }
    async write(data) {
        if (data && typeof data === "object" && data.type) {
            if (data.type === "seek") { this.__position = Number(data.position); return; }
            if (data.type === "truncate") { this.__bytes.length = Number(data.size); return; }
            data = data.data;
            if (data?.position !== undefined) this.__position = Number(data.position);
        }
        let bytes;
        if (typeof data === "string") bytes = Array.from(new TextEncoder().encode(data));
        else if (data instanceof Blob) bytes = Array.from(new Uint8Array(await data.arrayBuffer()));
        else bytes = Array.from(new Uint8Array(data.buffer ?? data, data.byteOffset ?? 0, data.byteLength));
        this.__bytes.splice(this.__position, bytes.length, ...bytes);
        this.__position += bytes.length;
    }
    seek(position) { this.__position = Number(position); return Promise.resolve(); }
    truncate(size) { this.__bytes.length = Number(size); return Promise.resolve(); }
    close() {
        save("opfs", this.__path, { bytes: this.__bytes, lastModified: Date.now() });
        __opfsWritableLocks.delete(this.__path);
        return Promise.resolve();
    }
    abort() { __opfsWritableLocks.delete(this.__path); return Promise.resolve(); }
}

class FileSystemDirectoryHandle {
    constructor(path = "") { this.kind = "directory"; this.name = path.split("/").pop() || ""; this.__path = path; }
    __child(name) {
        name = String(name);
        if (!name || name === "." || name === ".." || name.includes("/")) throw new TypeError("invalid file name");
        return this.__path ? `${this.__path}/${name}` : name;
    }
    getFileHandle(name, options = {}) {
        const path = this.__child(name);
        if (!options.create && call("persistentGet", "opfs", path) === null) return Promise.reject(new DOMException("File not found", "NotFoundError"));
        if (options.create && call("persistentGet", "opfs", path) === null) save("opfs", path, { bytes: [], lastModified: Date.now() });
        return Promise.resolve(new FileSystemFileHandle(path));
    }
    getDirectoryHandle(name, options = {}) {
        const path = this.__child(name);
        const exists = call("persistentGet", "opfs-directories", path) !== null;
        if (!exists && !options.create) return Promise.reject(new DOMException("Directory not found", "NotFoundError"));
        if (!exists) save("opfs-directories", path, { kind: "directory" });
        return Promise.resolve(new FileSystemDirectoryHandle(path));
    }
    removeEntry(name, options = {}) {
        const path = this.__child(name);
        const keys = JSON.parse(call("persistentList", "opfs"));
        const directories = JSON.parse(call("persistentList", "opfs-directories"));
        const matches = keys.filter(key => key === path || (options.recursive && key.startsWith(`${path}/`)));
        const directoryMatches = directories.filter(key => key === path || (options.recursive && key.startsWith(`${path}/`)));
        if (!matches.length && !directoryMatches.length) return Promise.reject(new DOMException("Entry not found", "NotFoundError"));
        for (const key of matches) call("persistentDelete", "opfs", key);
        for (const key of directoryMatches) call("persistentDelete", "opfs-directories", key);
        return Promise.resolve();
    }
    resolve(handle) { return Promise.resolve(handle.__path.startsWith(this.__path) ? handle.__path.slice(this.__path.length).split("/").filter(Boolean) : null); }
    async *entries() {
        const prefix = this.__path ? `${this.__path}/` : "";
        const files = JSON.parse(call("persistentList", "opfs")).filter(key => key.startsWith(prefix));
        const directories = JSON.parse(call("persistentList", "opfs-directories")).filter(key => key.startsWith(prefix));
        const names = new Map();
        for (const key of files) names.set(key.slice(prefix.length).split("/")[0], "file");
        for (const key of directories) names.set(key.slice(prefix.length).split("/")[0], "directory");
        for (const [name, kind] of names) yield [name, kind === "directory" ? new FileSystemDirectoryHandle(this.__child(name)) : new FileSystemFileHandle(this.__child(name))];
    }
    keys() { return (async function* (directory) { for await (const [name] of directory.entries()) yield name; })(this); }
    values() { return (async function* (directory) { for await (const [, value] of directory.entries()) yield value; })(this); }
    [Symbol.asyncIterator]() { return this.entries(); }
}

class StorageManager {
    estimate() { return Promise.resolve(JSON.parse(call("persistentEstimate"))); }
    persist() { return Promise.resolve(true); }
    persisted() { return Promise.resolve(true); }
    getDirectory() { return Promise.resolve(new FileSystemDirectoryHandle()); }
}

globalThis.IDBRequest = IDBRequest;
globalThis.IDBOpenDBRequest = IDBOpenDBRequest;
globalThis.IDBDatabase = IDBDatabase;
globalThis.IDBTransaction = IDBTransaction;
globalThis.IDBObjectStore = IDBObjectStore;
globalThis.IDBIndex = IDBIndex;
globalThis.IDBCursor = IDBCursor;
globalThis.IDBFactory = IDBFactory;
globalThis.indexedDB = new IDBFactory();
globalThis.Cache = Cache;
globalThis.CacheStorage = CacheStorage;
globalThis.caches = new CacheStorage();
globalThis.StorageManager = StorageManager;
globalThis.FileSystemDirectoryHandle = FileSystemDirectoryHandle;
globalThis.FileSystemFileHandle = FileSystemFileHandle;
globalThis.FileSystemWritableFileStream = FileSystemWritableFileStream;
Object.defineProperty(Navigator.prototype, "storage", { value: new StorageManager(), enumerable: true, configurable: true });

const legacyStorage = {
    queryUsageAndQuota(success) { const { usage, quota } = JSON.parse(call("persistentEstimate")); success?.(usage, quota); },
    requestQuota(bytes, success) { const { quota } = JSON.parse(call("persistentEstimate")); success?.(Math.min(Number(bytes), quota)); },
};
Object.defineProperty(Navigator.prototype, "webkitTemporaryStorage", { value: legacyStorage, enumerable: true, configurable: true });
Object.defineProperty(Navigator.prototype, "webkitPersistentStorage", { value: legacyStorage, enumerable: true, configurable: true });
for (const constructor of [IDBRequest, IDBOpenDBRequest, IDBDatabase, IDBTransaction, IDBObjectStore, IDBIndex, IDBCursor, IDBFactory, Cache, CacheStorage, StorageManager, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemWritableFileStream]) {
    globalThis.__brimpMarkWebBuiltin?.(constructor);
    for (const key of Reflect.ownKeys(constructor.prototype)) {
        const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
        if (typeof descriptor?.value === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.value);
        if (typeof descriptor?.get === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.get, `function get ${String(key)}() { [native code] }`);
    }
}
})();
