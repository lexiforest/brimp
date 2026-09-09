// Installs deterministic API-shape emulation from a fully resolved persona.
// Rendering, media processing, device access, and network services stay native
// or absent; this module only owns values that can be represented in JavaScript.
function installPersona(persona, runtimeFeatures) {
    "use strict";

    const navigatorValues = persona.navigator;
    const network = persona.base_headers;
    const markBuiltin = globalThis.__brimpMarkWebBuiltin;
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
    const defineNavigatorValue = (name, value) => {
        navigatorState[name] = value;
        markObjectFunctions(value);
        const getter = markAccessor(function () { return navigatorState[name]; }, "get", name);
        Object.defineProperty(Navigator.prototype, name, {
            get: getter,
            enumerable: true,
            configurable: true,
        });
    };

    for (const [name, value] of Object.entries({
        userAgent: network.user_agent,
        appVersion: network.user_agent.replace(/^Mozilla\//, ""),
        appCodeName: "Mozilla",
        appName: "Netscape",
        product: "Gecko",
        platform: navigatorValues.platform,
        language: persona.locale,
        languages: Object.freeze(persona.languages.slice()),
        hardwareConcurrency: navigatorValues.hardware_concurrency,
        maxTouchPoints: navigatorValues.max_touch_points,
        doNotTrack: navigatorValues.do_not_track || null,
        vendor: navigatorValues.vendor,
        productSub: navigatorValues.product_sub,
        pdfViewerEnabled: navigatorValues.pdf_viewer_enabled,
        cookieEnabled: true,
        onLine: true,
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

    const screenValues = persona.screen;
    class ScreenOrientation extends EventTarget {
        constructor(...args) {
            super();
            if (args[0] !== constructToken) throw new TypeError("Illegal constructor");
        }
        get type() { return screenValues.orientation_type; }
        get angle() { return screenValues.orientation_angle; }
    }
    const orientation = new ScreenOrientation(constructToken);
    class Screen {
        constructor() { throw new TypeError("Illegal constructor"); }
        get width() { return screenValues.width; }
        get height() { return screenValues.height; }
        get availWidth() { return screenValues.avail_width; }
        get availHeight() { return screenValues.avail_height; }
        get availLeft() { return screenValues.avail_left; }
        get availTop() { return screenValues.avail_top; }
        get left() { return screenValues.left; }
        get top() { return screenValues.top; }
        get colorDepth() { return screenValues.color_depth; }
        get pixelDepth() { return screenValues.pixel_depth; }
        get isExtended() { return screenValues.is_extended; }
        get orientation() { return orientation; }
    }
    const screenObject = Object.create(Screen.prototype);
    defineGlobalConstructor("Screen", Screen);
    defineGlobalConstructor("ScreenOrientation", ScreenOrientation);
    defineValue(globalThis, "screen", screenObject, false);

    const windowValues = persona.window;
    Object.defineProperties(globalThis, {
        outerWidth: { value: windowValues.outer_width, configurable: true },
        outerHeight: { value: windowValues.outer_height, configurable: true },
        screenX: { value: windowValues.screen_x, configurable: true },
        screenY: { value: windowValues.screen_y, configurable: true },
        screenLeft: { value: windowValues.screen_x, configurable: true },
        screenTop: { value: windowValues.screen_y, configurable: true },
    });

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

    if (!navigatorValues.offscreen_canvas_enabled) {
        delete globalThis.OffscreenCanvas;
        delete globalThis.OffscreenCanvasRenderingContext2D;
        delete HTMLCanvasElement.prototype.transferControlToOffscreen;
    }

    if (!runtimeFeatures.persistentStorage) {
        delete Navigator.prototype.storage;
        delete Navigator.prototype.webkitTemporaryStorage;
        delete Navigator.prototype.webkitPersistentStorage;
    }

    const applyIdentityOverride = serialized => {
        const override = JSON.parse(serialized);
        for (const name of ["userAgent", "platform", "language", "languages"]) {
            if (Object.hasOwn(override, name) && override[name] !== null) {
                navigatorState[name] = name === "languages"
                    ? Object.freeze(Array.from(override[name], String))
                    : String(override[name]);
            }
        }
        return true;
    };
    globalThis.__brimpFinalizeWebIdl();
    delete globalThis.__brimpFinalizeWebIdl;
    delete globalThis.__brimpMarkWebBuiltin;
    return applyIdentityOverride;
}
