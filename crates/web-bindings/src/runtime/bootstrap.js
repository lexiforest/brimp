(() => {
"use strict";

const __callHost = globalThis.__brimp;
delete globalThis.__brimp;
const __runtimeFeatures = JSON.parse(__callHost("runtimeFeatures"));

const __initialGlobalNames = new Set(Object.getOwnPropertyNames(globalThis));
const __nativeFunctions = new WeakSet();
const __nativeSources = new WeakMap();
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

Object.defineProperty(Function.prototype, "toString", {
    value: __functionToString,
    writable: true,
    enumerable: false,
    configurable: true,
});
__markWebBuiltin(__functionToString);

// Persona installation runs immediately after these bindings. Keep the marker
// non-enumerable during that hand-off; persona.js removes it before page code runs.
Object.defineProperty(globalThis, "__brimpMarkWebBuiltin", {
    value: __markWebBuiltin,
    writable: false,
    enumerable: false,
    configurable: true,
});

const __eventListeners = new WeakMap();
const __disconnectedNodeOrder = new WeakMap();
const __attributeMaps = new WeakMap();
const __inputValues = new WeakMap();
const __iframeBrowsingContexts = new WeakMap();
const __templateContents = new WeakMap();
let __activeElement = null;
let __nextDisconnectedNodeOrder = 1;

