(() => {
"use strict";

const __nativeHost = globalThis.__brimp;
const __domMutations = new Set(["setTextContent", "appendChild", "removeChild", "insertBefore", "setAttribute", "removeAttribute", "setInnerHTML", "styleSetCssText", "styleSetProperty", "styleRemoveProperty"]);
function __callHost(operation, ...args) {
    const before = __domMutations.has(operation) ? __mutationBefore(operation, args) : null;
    const iteratorUpdates = __domMutations.has(operation) ? __iteratorMutationBefore(operation, args) : [];
    const updateRanges = __domMutations.has(operation) ? __rangeMutationBefore(operation, args) : null;
    const result = __nativeHost(operation, ...args);
    if (updateRanges) updateRanges();
    for (const update of iteratorUpdates) update();
    if (before) __mutationAfter(before);
    if (__domMutations.has(operation)) __updateSlots();
    return result;
}
delete globalThis.__brimp;

// Capture the realm clock before any Web API captures Date.now.
if (__nativeHost("clockLogical")) {
    const OriginalDate = Date;
    const now = () => __nativeHost("clockOrigin") + __nativeHost("clockNow");
    function RealmDate(...args) {
        if (!new.target) return new OriginalDate(now()).toString();
        return Reflect.construct(OriginalDate, args.length ? args : [now()], new.target);
    }
    Object.setPrototypeOf(RealmDate, OriginalDate);
    RealmDate.prototype = OriginalDate.prototype;
    Object.defineProperty(RealmDate, "name", {value: "Date"});
    Object.defineProperty(RealmDate, "length", {value: 7});
    Object.defineProperty(RealmDate, "now", {value: now, configurable: true, writable: true});
    Object.defineProperty(RealmDate.prototype, "constructor", {value: RealmDate, configurable: true, writable: true});
    globalThis.Date = RealmDate;
}


const __initialGlobalNames = new Set(Object.getOwnPropertyNames(globalThis));
const __nativeFunctions = new WeakSet();
const __nativeSources = new WeakMap();
const __finalizedWebIdlPrototypes = new WeakSet();
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

function __nativeMemberSource(fn, key, kind) {
    const exactSource = __nativeSources.get(fn);
    if (exactSource !== undefined) return exactSource;
    const name = kind === "value" ? String(key) : `${kind} ${String(key)}`;
    return `function ${name}() { [native code] }`;
}

function __finalizeWebIdlInterfaces() {
    for (const name of Object.getOwnPropertyNames(globalThis)) {
        if (__initialGlobalNames.has(name)) continue;
        const constructor = Object.getOwnPropertyDescriptor(globalThis, name)?.value;
        if (typeof constructor !== "function" || constructor.prototype === undefined) continue;
        const prototype = constructor.prototype;
        if (prototype === null || typeof prototype !== "object" || __finalizedWebIdlPrototypes.has(prototype)) continue;
        __finalizedWebIdlPrototypes.add(prototype);
        if (!Object.hasOwn(prototype, Symbol.toStringTag)) {
            Object.defineProperty(prototype, Symbol.toStringTag, {
                value: name,
                writable: false,
                enumerable: false,
                configurable: true,
            });
        }
        for (const key of Reflect.ownKeys(prototype)) {
            if (key === "constructor") continue;
            const descriptor = Object.getOwnPropertyDescriptor(prototype, key);
            if (descriptor === undefined) continue;
            if (!descriptor.configurable) continue;
            for (const kind of ["value", "get", "set"]) {
                const fn = descriptor[kind];
                if (typeof fn !== "function") continue;
                const source = __nativeMemberSource(fn, key, kind);
                descriptor[kind] = new Proxy(fn, {
                    apply(target, receiver, argumentsList) {
                        const objectReceiver = (typeof receiver === "object" && receiver !== null)
                            || typeof receiver === "function";
                        const compatible = constructor === Window
                            ? receiver === globalThis
                            : objectReceiver && Object.prototype.isPrototypeOf.call(prototype, receiver);
                        if (!compatible) throw new TypeError("Illegal invocation");
                        return Reflect.apply(target, receiver, argumentsList);
                    },
                });
                __markWebBuiltin(descriptor[kind], source);
            }
            Object.defineProperty(prototype, key, descriptor);
        }
    }
    // Only run during trusted initialization. Inspect descriptors, never invoke
    // accessors, and preserve the descriptors/identity chosen by API installers.
    // A private graph walk includes namespace objects and static methods as well
    // as prototypes, so core and plugin APIs share the same source boundary.
    const visited = new WeakSet();
    function visit(object) {
        if (object === null || (typeof object !== 'object' && typeof object !== 'function') || visited.has(object)) return;
        visited.add(object);
        if (typeof object === 'function') __markWebBuiltin(object);
        for (const key of Reflect.ownKeys(object)) {
            const descriptor = Object.getOwnPropertyDescriptor(object, key);
            if (!descriptor) continue;
            if (typeof descriptor.get === 'function') __markWebBuiltin(descriptor.get, `function get ${String(key)}() { [native code] }`);
            if (typeof descriptor.set === 'function') __markWebBuiltin(descriptor.set, `function set ${String(key)}() { [native code] }`);
            if ('value' in descriptor) visit(descriptor.value);
        }
        visit(Object.getPrototypeOf(object));
    }
    visited.add(globalThis);
    for (const key of Reflect.ownKeys(globalThis)) {
        if (__initialGlobalNames.has(key)) continue;
        const descriptor = Object.getOwnPropertyDescriptor(globalThis, key);
        if ('value' in descriptor) visit(descriptor.value);
        else {
            if (descriptor.get) __markWebBuiltin(descriptor.get, `function get ${String(key)}() { [native code] }`);
            if (descriptor.set) __markWebBuiltin(descriptor.set, `function set ${String(key)}() { [native code] }`);
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

// Captured and removed by native initialization before plugin/page execution.
Object.defineProperty(globalThis, "__brimpFinalizeWebIdl", {
    value: __finalizeWebIdlInterfaces,
    writable: false,
    enumerable: false,
    configurable: true,
});

const __eventListeners = new WeakMap();
const __disconnectedNodeOrder = new WeakMap();
const __attributeMaps = new WeakMap();
const __inputValues = new WeakMap();
const __inputCheckedStates = new WeakMap();
const __optionSelectedStates = new WeakMap();
const __iframeBrowsingContexts = new WeakMap();
const __templateContents = new WeakMap();
let __activeElement = null;
let __nextDisconnectedNodeOrder = 1;
