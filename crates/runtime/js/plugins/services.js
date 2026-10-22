({global}, options = {}) => {
// Installs deterministic API-shape emulation from a fully resolved persona.
// Rendering, media processing, device access, and network services stay native
// or absent; this module only owns values that can be represented in JavaScript.
function installOptional(persona) {
    "use strict";

    const navigatorValues = persona.navigator;
    const network = persona.base_headers;
    const markBuiltin = () => {};
    const markNative = (fn, name = undefined) => {
        if (typeof fn === "function") {
            if (name === undefined) markBuiltin(fn);
            else markBuiltin(fn, `function ${name}() { [native code] }`);
        }
        return fn;
    };
    const markAccessor = (fn, kind, name) => {
        Object.defineProperty(fn, "name", { value: `${kind} ${name}`, configurable: true });
        markBuiltin(fn, `function ${kind} ${name}() { [native code] }`);
        return fn;
    };
    const markInterface = constructor => {
        markNative(constructor);
        if (!constructor.prototype) return constructor;
        if (!Object.hasOwn(constructor.prototype, Symbol.toStringTag)) {
            Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
                value: constructor.name,
                writable: false,
                enumerable: false,
                configurable: true,
            });
        }
        for (const key of Reflect.ownKeys(constructor.prototype)) {
            const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
            if (typeof descriptor?.value === "function") markNative(descriptor.value);
            if (typeof descriptor?.get === "function") markAccessor(descriptor.get, "get", String(key));
            if (typeof descriptor?.set === "function") markAccessor(descriptor.set, "set", String(key));
        }
        return constructor;
    };
    const markedObjects = new WeakSet();
    const markObjectFunctions = value => {
        if ((typeof value !== "object" && typeof value !== "function") || value === null) return;
        if (markedObjects.has(value)) return;
        markedObjects.add(value);
        for (const key of Reflect.ownKeys(value)) {
            const descriptor = Object.getOwnPropertyDescriptor(value, key);
            if (typeof descriptor?.value === "function") markNative(descriptor.value, String(key));
            else if (typeof descriptor?.value === "object") markObjectFunctions(descriptor.value);
            if (typeof descriptor?.get === "function") markAccessor(descriptor.get, "get", String(key));
            if (typeof descriptor?.set === "function") markAccessor(descriptor.set, "set", String(key));
        }
    };
    const defineValue = (target, name, value, enumerable = true) => {
        if (typeof value === "function") markInterface(value);
        else markObjectFunctions(value);
        Object.defineProperty(target, name, {
            value,
            enumerable,
            configurable: true,
        });
    };
    const defineGlobalConstructor = (name, constructor) => {
        markInterface(constructor);
        Object.defineProperty(globalThis, name, {
            value: constructor,
            writable: true,
            enumerable: false,
            configurable: true,
        });
    };
    const navigatorState = Object.create(null);
    const navigatorDefaults = Object.create(null);
    const defineNavigatorValue = (name, value) => {
        navigatorState[name] = value;
        navigatorDefaults[name] = value;
        markObjectFunctions(value);
        const getter = markAccessor(function () { return navigatorState[name]; }, "get", name);
        Object.defineProperty(Navigator.prototype, name, {
            get: getter,
            enumerable: true,
            configurable: true,
        });
    };

    for (const [name, value] of Object.entries({
        hardwareConcurrency: navigatorValues.hardware_concurrency,
        maxTouchPoints: navigatorValues.max_touch_points,
        vendor: navigatorValues.vendor,
        productSub: navigatorValues.product_sub,
        pdfViewerEnabled: navigatorValues.pdf_viewer_enabled,
    })) defineNavigatorValue(name, value);
    defineNavigatorValue("deviceMemory", navigatorValues.device_memory_gb);
    if (navigatorValues.expose_global_privacy_control) {
        defineNavigatorValue("globalPrivacyControl", navigatorValues.global_privacy_control);
    }

    const constructToken = Symbol("browser interface construction");
    class NetworkInformation extends EventTarget {
        constructor(...args) {
            super();
            if (args[0] !== constructToken) throw new TypeError("Illegal constructor");
        }
        get type() { return navigatorValues.connection_type; }
        get rtt() { return navigatorValues.connection_rtt_ms; }
        get downlink() { return Number(navigatorValues.connection_downlink_mbps); }
        get effectiveType() { return navigatorValues.connection_effective_type; }
        get saveData() { return navigatorValues.connection_save_data; }
    }
    defineNavigatorValue("connection", new NetworkInformation(constructToken));

    const parseBrands = header => {
        const result = [];
        const pattern = /"([^"]+)";v="([^"]+)"/g;
        for (let match; (match = pattern.exec(header)) !== null;) {
            result.push(Object.freeze({ brand: match[1], version: match[2] }));
        }
        return Object.freeze(result);
    };
    const unquote = value => value.replace(/^"|"$/g, "");
    if (network.sec_ch_ua) {
        const brands = parseBrands(network.sec_ch_ua);
        const fullVersionList = parseBrands(network.sec_ch_ua_full_version_list);
        class NavigatorUAData {
            constructor(...args) {
                if (args[0] !== constructToken) throw new TypeError("Illegal constructor");
            }
            get brands() { return brands; }
            get mobile() { return network.sec_ch_ua_mobile === "?1"; }
            get platform() { return unquote(network.sec_ch_ua_platform); }
            getHighEntropyValues(hints) {
                const values = {
                    architecture: unquote(network.sec_ch_ua_arch),
                    bitness: unquote(network.sec_ch_ua_bitness),
                    model: unquote(network.sec_ch_ua_model),
                    platformVersion: unquote(network.sec_ch_ua_platform_version),
                    uaFullVersion: unquote(network.sec_ch_ua_full_version),
                    fullVersionList,
                    wow64: false,
                };
                const selected = this.toJSON();
                for (const hint of hints) {
                    if (Object.hasOwn(values, hint)) selected[hint] = values[hint];
                }
                return Promise.resolve(selected);
            }
            toJSON() {
                return { brands, mobile: this.mobile, platform: this.platform };
            }
        }
        defineGlobalConstructor("NavigatorUAData", NavigatorUAData);
        defineNavigatorValue("userAgentData", new NavigatorUAData(constructToken));
    }

    const arrayData = new WeakMap();
    class MimeType {
        constructor() { throw new TypeError("Illegal constructor"); }
        get type() { return arrayData.get(this).type; }
        get suffixes() { return arrayData.get(this).suffixes; }
        get description() { return arrayData.get(this).description; }
        get enabledPlugin() { return arrayData.get(this).enabledPlugin; }
    }
    class Plugin {
        constructor() { throw new TypeError("Illegal constructor"); }
        get name() { return arrayData.get(this).name; }
        get filename() { return arrayData.get(this).filename; }
        get description() { return arrayData.get(this).description; }
        get length() { return arrayData.get(this).items.length; }
        item(index) { return arrayData.get(this).items[Number(index)] ?? null; }
        namedItem(name) { return arrayData.get(this).names[String(name)] ?? null; }
        [Symbol.iterator]() { return arrayData.get(this).items[Symbol.iterator](); }
    }
    class PluginArray {
        constructor() { throw new TypeError("Illegal constructor"); }
        get length() { return arrayData.get(this).items.length; }
        item(index) { return arrayData.get(this).items[Number(index)] ?? null; }
        namedItem(name) { return arrayData.get(this).names[String(name)] ?? null; }
        refresh() {}
        [Symbol.iterator]() { return arrayData.get(this).items[Symbol.iterator](); }
    }
    class MimeTypeArray {
        constructor() { throw new TypeError("Illegal constructor"); }
        get length() { return arrayData.get(this).items.length; }
        item(index) { return arrayData.get(this).items[Number(index)] ?? null; }
        namedItem(name) { return arrayData.get(this).names[String(name)] ?? null; }
        [Symbol.iterator]() { return arrayData.get(this).items[Symbol.iterator](); }
    }
    const makeArrayLike = (constructor, items, names) => {
        const value = Object.create(constructor.prototype);
        arrayData.set(value, { items, names });
        items.forEach((item, index) => defineValue(value, index, item));
        for (const [name, item] of Object.entries(names)) {
            if (!(name in value)) defineValue(value, name, item, false);
        }
        return value;
    };
    const mimeByType = Object.create(null);
    const pluginByName = Object.create(null);
    const pdfPlugins = navigatorValues.pdf_viewer_enabled ? [
        "PDF Viewer", "Chrome PDF Viewer", "Chromium PDF Viewer",
        "Microsoft Edge PDF Viewer", "WebKit built-in PDF",
    ] : [];
    const pluginEntries = pdfPlugins.map(name => {
        const pluginConfig = {
            name,
            filename: "internal-pdf-viewer",
            description: "Portable Document Format",
            mime_types: ["application/pdf", "text/pdf"].map(type => ({
                type, suffixes: "pdf", description: "Portable Document Format",
            })),
        };
        const plugin = Object.create(Plugin.prototype);
        const mimeNames = Object.create(null);
        const mimeEntries = pluginConfig.mime_types.map(mime => {
            const value = Object.create(MimeType.prototype);
            arrayData.set(value, { ...mime, enabledPlugin: plugin });
            mimeNames[mime.type] = value;
            if (!(mime.type in mimeByType)) mimeByType[mime.type] = value;
            return value;
        });
        arrayData.set(plugin, {
            name: pluginConfig.name,
            filename: pluginConfig.filename,
            description: pluginConfig.description,
            items: mimeEntries,
            names: mimeNames,
        });
        mimeEntries.forEach((item, index) => defineValue(plugin, index, item));
        pluginByName[pluginConfig.name] = plugin;
        return plugin;
    });
    const plugins = makeArrayLike(PluginArray, pluginEntries, pluginByName);
    const mimeTypes = makeArrayLike(MimeTypeArray, Object.values(mimeByType), mimeByType);
    for (const constructor of [MimeType, Plugin, PluginArray, MimeTypeArray]) {
        defineGlobalConstructor(constructor.name, constructor);
    }
    defineNavigatorValue("plugins", plugins);
    defineNavigatorValue("mimeTypes", mimeTypes);

    {
        function Notification(title, options = {}) {
            this.title = String(title);
            this.body = options.body === undefined ? "" : String(options.body);
            this.close = () => undefined;
        }
        Object.defineProperty(Notification, "permission", {
            get: () => navigatorValues.notification_permission,
        });
        Notification.requestPermission = callback => {
            const permission = navigatorValues.notification_permission;
            if (typeof callback === "function") callback(permission);
            return Promise.resolve(permission);
        };
        defineGlobalConstructor("Notification", Notification);
    }

    if (navigatorValues.permissions_enabled) {
        const permissions = Object.freeze({
            query(descriptor) {
                const state = descriptor && descriptor.name === "notifications"
                    ? (navigatorValues.notification_permission === "default" ? "prompt" : navigatorValues.notification_permission)
                    : "prompt";
                return Promise.resolve(Object.freeze({
                    state,
                    onchange: null,
                    addEventListener() {},
                    removeEventListener() {},
                }));
            },
        });
        defineNavigatorValue("permissions", permissions);
    } else {
        delete Navigator.prototype.permissions;
    }

    if (navigatorValues.bluetooth_enabled) {
        defineNavigatorValue("bluetooth", Object.freeze({
            getAvailability: () => Promise.resolve(navigatorValues.bluetooth_available),
        }));
    } else {
        delete Navigator.prototype.bluetooth;
    }

    if (navigatorValues.media_devices_enabled) {
        defineNavigatorValue("mediaDevices", Object.freeze({
            enumerateDevices: () => Promise.resolve([]),
            getSupportedConstraints: () => Object.freeze({}),
        }));
    } else {
        delete Navigator.prototype.mediaDevices;
    }

}
const defaults = {hardware_concurrency: 4, max_touch_points: 0, vendor: '', product_sub: '20030107', pdf_viewer_enabled: false, device_memory_gb: 8, connection_type: 'unknown', connection_rtt_ms: 0, connection_downlink_mbps: 0, connection_effective_type: '4g', connection_save_data: false, notification_permission: 'denied', permissions_enabled: true, bluetooth_enabled: true, bluetooth_available: false, media_devices_enabled: true};
installOptional({navigator: {...defaults, ...options}, base_headers: {sec_ch_ua: ''}});
const __navigatorServiceConstructionToken = Symbol();
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
    locks: { get() { return __lockManager; }, enumerable: true, configurable: true },
});


Object.assign(global, {Lock, LockManager});
return () => { for (const entry of [...__heldLocks, ...__pendingLocks]) __abortLock(entry, new DOMException("Plugin disposed", "AbortError")); };
}
